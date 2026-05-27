use crate::neurons::Neuron;
use crate::types::generalised_logistic_function;
use crate::Vote;
use std::collections::HashMap;

const DELEGATED_VOTE_DENOMINATOR: i32 = 2;
const FIXED_POINT_SCALING_FACTOR: i32 = 100; // *10 to mitigate float precission loss, and *10 to allow integer division
const ZERO_SNAP_THRESHOLD: f64 = 0.001; // float-noise floor: |output| <= this collapses to 0
#[derive(Clone, Debug)]
pub struct RetroVoteQualityNeuron {
    votes_per_round: HashMap<u32, HashMap<String, HashMap<String, Vote>>>, // round -> submission -> user -> vote (Yes/No/Abstain/Delegate)
    normalized_votes_per_round: HashMap<u32, HashMap<String, HashMap<String, Vote>>>, // round -> submission -> user -> vote (Yes/No/Abstain)
    tranche_status_map: HashMap<String, Vec<String>>, // tranche status -> [submission id (airtable)]
    submissions_airtable_ids: HashMap<String, String>,
}

impl RetroVoteQualityNeuron {
    pub fn from_data(
        votes_per_round: HashMap<u32, HashMap<String, HashMap<String, Vote>>>,
        normalized_votes_per_round: HashMap<u32, HashMap<String, HashMap<String, Vote>>>,
        tranche_status_map: HashMap<String, Vec<String>>,
        submissions_airtable_ids: HashMap<String, String>,
    ) -> Self {
        Self {
            votes_per_round,
            normalized_votes_per_round,
            tranche_status_map,
            submissions_airtable_ids,
        }
    }
    fn run_user(&self, user: &str) -> f64 {
        let mut total_bonus: i32 = 0;
        // loop through rounds
        for (round, round_votes) in &self.votes_per_round {
            // loop through all submissions
            for (submission_name, submission_votes) in round_votes {
                // loop through all votes
                for (voter, vote) in submission_votes {
                    // skip votes from other users, and no/abstain
                    if voter != user || vote == &Vote::No || vote == &Vote::Abstain {
                        continue;
                    };
                    // lookup bonus for this submission
                    let bonus_value: i32 = match self.lookup_tranche_status(&submission_name) {
                        Some(tranche_status) => tranche_status_to_bonus(&tranche_status),
                        None => continue,
                    };
                    match vote {
                        // apply bonus value
                        Vote::Yes => total_bonus += bonus_value,
                        // or resolve delegation
                        Vote::Delegate => {
                            // lookup this round-submission-user vote in normalized_votes_per_round
                            if let Some(resolved_vote) =
                                self.resolve_delegated_vote(*round, &submission_name, user)
                            {
                                // apply bonus value * 0.5
                                if resolved_vote == Vote::Yes {
                                    total_bonus += bonus_value / DELEGATED_VOTE_DENOMINATOR;
                                }
                            }
                        }
                        Vote::Abstain | Vote::No => {}
                    }
                }
            }
        }
        let raw_bonus = total_bonus as f64 / FIXED_POINT_SCALING_FACTOR as f64;
        // Mirror the curve around 0: run |raw| through the logistic (baseline-shifted
        // so raw=0 maps to 0) and flip the sign for negative raw bonuses, so penalties
        // produce symmetric negative scores instead of being clipped at the a=0 floor.
        let magnitude = logistic(raw_bonus.abs()) - logistic(0.0);
        let signed = if raw_bonus < 0.0 { -magnitude } else { magnitude };
        if signed.abs() <= ZERO_SNAP_THRESHOLD { 0.0 } else { signed }
    }
    fn resolve_delegated_vote(
        &self,
        round: u32,
        submission_name: &str,
        user: &str,
    ) -> Option<Vote> {
        let round_votes = match self.normalized_votes_per_round.get(&round) {
            Some(round_votes) => round_votes,
            None => {
                return None;
            }
        };
        let submission_votes = match round_votes.get(submission_name) {
            Some(submission_votes) => submission_votes,
            None => return None,
        };
        match submission_votes.get(user) {
            Some(vote) => return Some(vote.clone()),
            None => return None,
        }
    }
    fn lookup_tranche_status(&self, submission_name: &str) -> Option<String> {
        // lookup airtable id of the submission
        if let Some(airtable_id) = self.submissions_airtable_ids.get(submission_name) {
            // lookup tranche status
            for (status, airtable_ids) in &self.tranche_status_map {
                if airtable_ids.contains(airtable_id) {
                    return Some(status.to_string());
                }
            }
        }
        None
    }
}
fn logistic(raw_bonus: f64) -> f64 {
    generalised_logistic_function(0.0, 5.0, 1.0, 4.0, 1.0, 1.0, 1.0, raw_bonus)
}
fn tranche_status_to_bonus(tranche_status: &str) -> i32 {
    match tranche_status {
        "Live on Stellar within 6 months" => 30,               // 0.3
        "Live on Stellar after 6 months" => 10,                // 0.1
        "Not live on Stellar within 6 months, Awarded" => -30, // -0.3
        "Not live on Stellar within 6 months, MVP" => -20,     // -0.2
        "Not live on Stellar within 6 months, Testnet" => -10, // -0.1
        _ => 0,
    }
}
impl Neuron for RetroVoteQualityNeuron {
    fn name(&self) -> String {
        "retro_vote_quality_neuron".to_string()
    }

    fn calculate_result(&self, users: &[String]) -> HashMap<String, f64> {
        let mut result = HashMap::new();

        for user in users {
            let bonus = self.run_user(user);
            result.insert(user.into(), bonus);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIVE_WITHIN_6: &str = "Live on Stellar within 6 months";
    const LIVE_AFTER_6: &str = "Live on Stellar after 6 months";
    const NOT_LIVE_AWARDED: &str = "Not live on Stellar within 6 months, Awarded";
    const NOT_LIVE_MVP: &str = "Not live on Stellar within 6 months, MVP";
    const NOT_LIVE_TESTNET: &str = "Not live on Stellar within 6 months, Testnet";

    const FLOAT_EPS: f64 = 1e-12;

    fn logistic_of(raw_bonus: f64) -> f64 {
        // Mirrors the production formula: logistic(|raw|) - logistic(0), negated
        // for negative raw, so raw=0 maps to 0 and penalties stay symmetric.
        let f = |x| generalised_logistic_function(0.0, 5.0, 1.0, 4.0, 1.0, 1.0, 1.0, x);
        let magnitude = f(raw_bonus.abs()) - f(0.0);
        if raw_bonus < 0.0 { -magnitude } else { magnitude }
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < FLOAT_EPS, "expected {expected}, got {actual}");
    }

    fn build_neuron(
        votes_per_round: HashMap<u32, HashMap<String, HashMap<String, Vote>>>,
        normalized_votes_per_round: HashMap<u32, HashMap<String, HashMap<String, Vote>>>,
        submissions_to_status: &[(&str, &str, &str)], // (submission_name, airtable_id, status)
    ) -> RetroVoteQualityNeuron {
        let mut tranche_status_map: HashMap<String, Vec<String>> = HashMap::new();
        let mut submissions_airtable_ids: HashMap<String, String> = HashMap::new();
        for (name, airtable_id, status) in submissions_to_status {
            submissions_airtable_ids.insert((*name).to_string(), (*airtable_id).to_string());
            tranche_status_map
                .entry((*status).to_string())
                .or_insert_with(Vec::new)
                .push((*airtable_id).to_string());
        }
        RetroVoteQualityNeuron::from_data(
            votes_per_round,
            normalized_votes_per_round,
            tranche_status_map,
            submissions_airtable_ids,
        )
    }

    fn votes(
        round: u32,
        submission: &str,
        per_user: &[(&str, Vote)],
    ) -> HashMap<u32, HashMap<String, HashMap<String, Vote>>> {
        let users: HashMap<String, Vote> =
            per_user.iter().map(|(u, v)| ((*u).to_string(), v.clone())).collect();
        let submissions: HashMap<String, HashMap<String, Vote>> =
            HashMap::from([(submission.to_string(), users)]);
        HashMap::from([(round, submissions)])
    }

    #[test]
    fn tranche_status_to_bonus_known_values() {
        assert_eq!(tranche_status_to_bonus(LIVE_WITHIN_6), 30);
        assert_eq!(tranche_status_to_bonus(LIVE_AFTER_6), 10);
        assert_eq!(tranche_status_to_bonus(NOT_LIVE_AWARDED), -30);
        assert_eq!(tranche_status_to_bonus(NOT_LIVE_MVP), -20);
        assert_eq!(tranche_status_to_bonus(NOT_LIVE_TESTNET), -10);
    }

    #[test]
    fn tranche_status_to_bonus_unknown_returns_zero() {
        assert_eq!(tranche_status_to_bonus("anything else"), 0);
        assert_eq!(tranche_status_to_bonus(""), 0);
    }

    #[test]
    fn yes_vote_adds_full_bonus() {
        let neuron = build_neuron(
            votes(30, "sub1", &[("alice", Vote::Yes)]),
            HashMap::new(),
            &[("sub1", "rec1", LIVE_WITHIN_6)],
        );
        assert_close(neuron.run_user("alice"), logistic_of(0.30));
    }

    #[test]
    fn no_and_abstain_votes_contribute_nothing() {
        let neuron = build_neuron(
            votes(30, "sub1", &[("alice", Vote::No), ("bob", Vote::Abstain)]),
            HashMap::new(),
            &[("sub1", "rec1", LIVE_WITHIN_6)],
        );
        let baseline = logistic_of(0.0);
        assert_close(neuron.run_user("alice"), baseline);
        assert_close(neuron.run_user("bob"), baseline);
    }

    #[test]
    fn delegate_resolving_to_yes_adds_half_bonus() {
        let neuron = build_neuron(
            votes(30, "sub1", &[("alice", Vote::Delegate)]),
            votes(30, "sub1", &[("alice", Vote::Yes)]),
            &[("sub1", "rec1", LIVE_WITHIN_6)],
        );
        // raw bonus: 30 / 2 = 15 -> 0.15, then logistic
        assert_close(neuron.run_user("alice"), logistic_of(0.15));
    }

    #[test]
    fn delegate_resolving_to_yes_with_negative_status_halves_penalty_via_int_division() {
        // raw bonus: -30 / 2 = -15 -> -0.15 (Rust integer division truncates toward zero)
        let neuron = build_neuron(
            votes(30, "sub1", &[("alice", Vote::Delegate)]),
            votes(30, "sub1", &[("alice", Vote::Yes)]),
            &[("sub1", "rec1", NOT_LIVE_AWARDED)],
        );
        assert_close(neuron.run_user("alice"), logistic_of(-0.15));
    }

    #[test]
    fn delegate_resolving_to_no_or_abstain_contributes_nothing() {
        let neuron_no = build_neuron(
            votes(30, "sub1", &[("alice", Vote::Delegate)]),
            votes(30, "sub1", &[("alice", Vote::No)]),
            &[("sub1", "rec1", LIVE_WITHIN_6)],
        );
        let neuron_abstain = build_neuron(
            votes(30, "sub1", &[("alice", Vote::Delegate)]),
            votes(30, "sub1", &[("alice", Vote::Abstain)]),
            &[("sub1", "rec1", LIVE_WITHIN_6)],
        );
        let baseline = logistic_of(0.0);
        assert_close(neuron_no.run_user("alice"), baseline);
        assert_close(neuron_abstain.run_user("alice"), baseline);
    }

    #[test]
    fn delegate_unresolved_in_normalized_votes_contributes_nothing() {
        // No entry at all in normalized_votes_per_round
        let neuron_missing_round = build_neuron(
            votes(30, "sub1", &[("alice", Vote::Delegate)]),
            HashMap::new(),
            &[("sub1", "rec1", LIVE_WITHIN_6)],
        );
        // Round present but submission missing
        let neuron_missing_submission = build_neuron(
            votes(30, "sub1", &[("alice", Vote::Delegate)]),
            votes(30, "other_sub", &[("alice", Vote::Yes)]),
            &[("sub1", "rec1", LIVE_WITHIN_6)],
        );
        // Submission present but user missing
        let neuron_missing_user = build_neuron(
            votes(30, "sub1", &[("alice", Vote::Delegate)]),
            votes(30, "sub1", &[("bob", Vote::Yes)]),
            &[("sub1", "rec1", LIVE_WITHIN_6)],
        );
        let baseline = logistic_of(0.0);
        assert_close(neuron_missing_round.run_user("alice"), baseline);
        assert_close(neuron_missing_submission.run_user("alice"), baseline);
        assert_close(neuron_missing_user.run_user("alice"), baseline);
    }

    #[test]
    fn submission_with_unknown_tranche_status_is_skipped() {
        // submission registered, but its airtable id is in no status list
        let mut submissions_airtable_ids = HashMap::new();
        submissions_airtable_ids.insert("sub1".to_string(), "rec1".to_string());
        let neuron = RetroVoteQualityNeuron::from_data(
            votes(30, "sub1", &[("alice", Vote::Yes)]),
            HashMap::new(),
            HashMap::new(), // empty tranche_status_map
            submissions_airtable_ids,
        );
        assert_close(neuron.run_user("alice"), logistic_of(0.0));
    }

    #[test]
    fn submission_without_airtable_id_is_skipped() {
        // submission has a Yes vote but no airtable id -> lookup fails -> skip
        let neuron = RetroVoteQualityNeuron::from_data(
            votes(30, "sub1", &[("alice", Vote::Yes)]),
            HashMap::new(),
            HashMap::from([(LIVE_WITHIN_6.to_string(), vec!["rec1".to_string()])]),
            HashMap::new(), // sub1 has no airtable_id mapping
        );
        assert_close(neuron.run_user("alice"), logistic_of(0.0));
    }

    #[test]
    fn negative_tranche_status_subtracts_bonus() {
        let neuron = build_neuron(
            votes(30, "sub1", &[("alice", Vote::Yes)]),
            HashMap::new(),
            &[("sub1", "rec1", NOT_LIVE_MVP)],
        );
        assert_close(neuron.run_user("alice"), logistic_of(-0.20));
    }

    #[test]
    fn other_users_votes_are_ignored() {
        let neuron = build_neuron(
            votes(30, "sub1", &[("alice", Vote::Yes), ("bob", Vote::Yes)]),
            HashMap::new(),
            &[("sub1", "rec1", LIVE_WITHIN_6)],
        );
        let yes_bonus = logistic_of(0.30);
        let baseline = logistic_of(0.0);
        assert_close(neuron.run_user("alice"), yes_bonus);
        assert_close(neuron.run_user("bob"), yes_bonus);
        assert_close(neuron.run_user("carol"), baseline);
    }

    #[test]
    fn bonuses_accumulate_across_rounds_and_submissions() {
        // round 30: sub1 Yes (+0.30), sub2 Delegate->Yes (+0.05, half of 0.10)
        // round 31: sub3 Yes (-0.30 awarded), sub4 Yes (-0.10 testnet)
        // raw total = 0.30 + 0.05 - 0.30 - 0.10 = -0.05, then logistic
        let mut votes_per_round: HashMap<u32, HashMap<String, HashMap<String, Vote>>> =
            HashMap::new();
        votes_per_round.insert(
            30,
            HashMap::from([
                ("sub1".to_string(), HashMap::from([("alice".to_string(), Vote::Yes)])),
                ("sub2".to_string(), HashMap::from([("alice".to_string(), Vote::Delegate)])),
            ]),
        );
        votes_per_round.insert(
            31,
            HashMap::from([
                ("sub3".to_string(), HashMap::from([("alice".to_string(), Vote::Yes)])),
                ("sub4".to_string(), HashMap::from([("alice".to_string(), Vote::Yes)])),
            ]),
        );
        let normalized = votes(30, "sub2", &[("alice", Vote::Yes)]);
        let neuron = build_neuron(
            votes_per_round,
            normalized,
            &[
                ("sub1", "rec1", LIVE_WITHIN_6),
                ("sub2", "rec2", LIVE_AFTER_6),
                ("sub3", "rec3", NOT_LIVE_AWARDED),
                ("sub4", "rec4", NOT_LIVE_TESTNET),
            ],
        );
        assert_close(neuron.run_user("alice"), logistic_of(-0.05));
    }

    #[test]
    fn empty_data_returns_logistic_of_zero() {
        let neuron = build_neuron(HashMap::new(), HashMap::new(), &[]);
        assert_close(neuron.run_user("alice"), logistic_of(0.0));
    }

    #[test]
    fn logistic_parameters_pinned() {
        // Locks in the (a=0, k=5, c=1, q=4, b=1, nu=1, x_off=1) configuration so
        // accidental parameter changes are caught even if `logistic_of` is updated.
        // Raw logistic(0) = 5 / (1 + 4 * exp(1)) ≈ 0.421119042004487; production
        // subtracts that baseline so a voter with no contributions scores 0.
        let neuron = build_neuron(HashMap::new(), HashMap::new(), &[]);
        let result = neuron.run_user("alice");
        assert!(result.abs() < 1e-12, "expected 0 for empty contributions, got {result}");
        let raw_baseline = generalised_logistic_function(0.0, 5.0, 1.0, 4.0, 1.0, 1.0, 1.0, 0.0);
        assert!(
            (raw_baseline - 0.421_119_042_004_487).abs() < 1e-12,
            "logistic baseline drifted: got {raw_baseline}"
        );
    }

    #[test]
    fn logistic_is_monotonic_and_bounded() {
        let mk = |sub_count: usize, status: &str| {
            // build N submissions all with the same status, alice voting Yes on each
            let mut sub_votes: HashMap<String, HashMap<String, Vote>> = HashMap::new();
            let mut subs_to_status: Vec<(String, String, String)> = Vec::new();
            for i in 0..sub_count {
                let name = format!("sub{i}");
                let rec = format!("rec{i}");
                sub_votes.insert(name.clone(), HashMap::from([("alice".to_string(), Vote::Yes)]));
                subs_to_status.push((name, rec, status.to_string()));
            }
            let votes_per_round = HashMap::from([(30u32, sub_votes)]);
            let refs: Vec<(&str, &str, &str)> = subs_to_status
                .iter()
                .map(|(n, r, s)| (n.as_str(), r.as_str(), s.as_str()))
                .collect();
            build_neuron(votes_per_round, HashMap::new(), &refs).run_user("alice")
        };
        // More positive Yes votes => higher score, asymptoting at k=5.
        let one = mk(1, LIVE_WITHIN_6);
        let many = mk(50, LIVE_WITHIN_6); // raw = 50 * 0.30 = 15
        let huge = mk(500, LIVE_WITHIN_6); // raw is large enough that f64 saturates at 5.0
        assert!(one < many, "one={one} many={many}");
        assert!(many <= huge, "many={many} huge={huge}");
        // Bounded above by k=5 minus the baseline subtraction (~0.421).
        assert!(huge <= 5.0, "huge={huge}");
        assert!(many > 4.0, "many={many}"); // logistic_of(15) is already very close to k=5
                                            // Penalties mirror the positive side: bounded below by ~-(k - baseline).
        let very_negative = mk(50, NOT_LIVE_AWARDED); // raw = 50 * -0.30 = -15
        assert!(very_negative >= -5.0, "very_negative={very_negative}");
        assert!(very_negative < -4.0, "very_negative={very_negative}");
    }

    #[test]
    fn calculate_result_returns_entry_for_every_user() {
        let neuron = build_neuron(
            votes(30, "sub1", &[("alice", Vote::Yes), ("bob", Vote::No)]),
            HashMap::new(),
            &[("sub1", "rec1", LIVE_WITHIN_6)],
        );
        let users = vec!["alice".to_string(), "bob".to_string(), "carol".to_string()];
        let result = neuron.calculate_result(&users);
        assert_eq!(result.len(), 3);
        let baseline = logistic_of(0.0);
        assert_close(*result.get("alice").unwrap(), logistic_of(0.30));
        assert_close(*result.get("bob").unwrap(), baseline);
        assert_close(*result.get("carol").unwrap(), baseline);
    }

    #[test]
    fn neuron_name_is_stable() {
        let neuron = build_neuron(HashMap::new(), HashMap::new(), &[]);
        assert_eq!(neuron.name(), "retro_vote_quality_neuron");
    }
}
