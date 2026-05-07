use crate::neurons::Neuron;
use crate::Vote;
use std::collections::HashMap;

const DELEGATED_VOTE_DENOMINATOR: i32 = 2;
const FIXED_POINT_SCALING_FACTOR: i32 = 100; // *10 to mitigate float precission loss, and *10 to allow integer division
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
        total_bonus as f64 / FIXED_POINT_SCALING_FACTOR as f64
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

    // Build a neuron where each submission is registered under a single status so
    // `lookup_tranche_status` is deterministic regardless of HashMap iteration order.
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
        assert_eq!(neuron.run_user("alice"), 0.30);
    }

    #[test]
    fn no_and_abstain_votes_contribute_nothing() {
        let neuron = build_neuron(
            votes(30, "sub1", &[("alice", Vote::No), ("bob", Vote::Abstain)]),
            HashMap::new(),
            &[("sub1", "rec1", LIVE_WITHIN_6)],
        );
        assert_eq!(neuron.run_user("alice"), 0.0);
        assert_eq!(neuron.run_user("bob"), 0.0);
    }

    #[test]
    fn delegate_resolving_to_yes_adds_half_bonus() {
        let neuron = build_neuron(
            votes(30, "sub1", &[("alice", Vote::Delegate)]),
            votes(30, "sub1", &[("alice", Vote::Yes)]),
            &[("sub1", "rec1", LIVE_WITHIN_6)],
        );
        // 30 / 2 = 15 -> 0.15
        assert_eq!(neuron.run_user("alice"), 0.15);
    }

    #[test]
    fn delegate_resolving_to_yes_with_negative_status_halves_penalty_via_int_division() {
        // -30 / 2 = -15 -> -0.15 (integer division on negatives in Rust truncates toward zero)
        let neuron = build_neuron(
            votes(30, "sub1", &[("alice", Vote::Delegate)]),
            votes(30, "sub1", &[("alice", Vote::Yes)]),
            &[("sub1", "rec1", NOT_LIVE_AWARDED)],
        );
        assert_eq!(neuron.run_user("alice"), -0.15);
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
        assert_eq!(neuron_no.run_user("alice"), 0.0);
        assert_eq!(neuron_abstain.run_user("alice"), 0.0);
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
        assert_eq!(neuron_missing_round.run_user("alice"), 0.0);
        assert_eq!(neuron_missing_submission.run_user("alice"), 0.0);
        assert_eq!(neuron_missing_user.run_user("alice"), 0.0);
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
        assert_eq!(neuron.run_user("alice"), 0.0);
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
        assert_eq!(neuron.run_user("alice"), 0.0);
    }

    #[test]
    fn negative_tranche_status_subtracts_bonus() {
        let neuron = build_neuron(
            votes(30, "sub1", &[("alice", Vote::Yes)]),
            HashMap::new(),
            &[("sub1", "rec1", NOT_LIVE_MVP)],
        );
        assert_eq!(neuron.run_user("alice"), -0.20);
    }

    #[test]
    fn other_users_votes_are_ignored() {
        let neuron = build_neuron(
            votes(30, "sub1", &[("alice", Vote::Yes), ("bob", Vote::Yes)]),
            HashMap::new(),
            &[("sub1", "rec1", LIVE_WITHIN_6)],
        );
        assert_eq!(neuron.run_user("alice"), 0.30);
        assert_eq!(neuron.run_user("bob"), 0.30);
        assert_eq!(neuron.run_user("carol"), 0.0);
    }

    #[test]
    fn bonuses_accumulate_across_rounds_and_submissions() {
        // round 30: sub1 Yes (+0.30), sub2 Delegate->Yes (+0.05, half of 0.10)
        // round 31: sub3 Yes (-0.30 awarded), sub4 Yes (-0.10 testnet)
        // total = 0.30 + 0.05 - 0.30 - 0.10 = -0.05
        let mut votes_per_round: HashMap<u32, HashMap<String, HashMap<String, Vote>>> =
            HashMap::new();
        votes_per_round.insert(
            30,
            HashMap::from([
                (
                    "sub1".to_string(),
                    HashMap::from([("alice".to_string(), Vote::Yes)]),
                ),
                (
                    "sub2".to_string(),
                    HashMap::from([("alice".to_string(), Vote::Delegate)]),
                ),
            ]),
        );
        votes_per_round.insert(
            31,
            HashMap::from([
                (
                    "sub3".to_string(),
                    HashMap::from([("alice".to_string(), Vote::Yes)]),
                ),
                (
                    "sub4".to_string(),
                    HashMap::from([("alice".to_string(), Vote::Yes)]),
                ),
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
        let result = neuron.run_user("alice");
        // Use a tolerance in case of f64 rounding from int->f64 division
        assert!((result - (-0.05)).abs() < 1e-9, "got {result}");
    }

    #[test]
    fn run_user_with_empty_data_returns_zero() {
        let neuron = build_neuron(HashMap::new(), HashMap::new(), &[]);
        assert_eq!(neuron.run_user("alice"), 0.0);
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
        assert_eq!(result.get("alice"), Some(&0.30));
        assert_eq!(result.get("bob"), Some(&0.0));
        assert_eq!(result.get("carol"), Some(&0.0));
    }

    #[test]
    fn neuron_name_is_stable() {
        let neuron = build_neuron(HashMap::new(), HashMap::new(), &[]);
        assert_eq!(neuron.name(), "retro_vote_quality_neuron");
    }
}
