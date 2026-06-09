use crate::neurons::Neuron;
use crate::types::generalised_logistic_function;
use crate::Vote;
use serde::Deserialize;
use std::collections::HashMap;
// use wasm_bindgen::JsValue;
// use web_sys::{self, console};
const ROUND_IMPORTANCE_DECAY_OFFSET: u32 = 8;
const ACTIVE_VOTES_HISTORY_OLDEST_ROUND: u32 = 32; // we dont have data from rounds before 32
const ACTIVE_VOTES_MIN_RATIO: f64 = 0.5; // lowest possible ratio of active votes

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug)]
pub struct PriorVotingHistoryNeuron {
    users_round_history: HashMap<String, Vec<u32>>,
    votes_per_round: HashMap<u32, HashMap<String, HashMap<String, Vote>>>, // round -> submission -> user -> vote
    current_round: u32,
}

impl PriorVotingHistoryNeuron {
    pub fn from_data(
        users_round_history: HashMap<String, Vec<u32>>,
        votes_per_round: HashMap<u32, HashMap<String, HashMap<String, Vote>>>,
        current_round: u32,
    ) -> Self {
        Self {
            users_round_history,
            votes_per_round,
            current_round,
        }
    }

    /// Computes the prior-voting-history bonus for a single user.
    ///
    /// # Panics
    ///
    /// Panics with `"history bonus offset is bigger than current round"` if `current_round` is
    /// smaller than `ROUND_IMPORTANCE_DECAY_OFFSET` (for example after a round reset to 0), which
    /// would otherwise underflow the round-importance offset subtraction.
    pub fn calculate_bonus(&self, user: String) -> f64 {
        // console::log_1(&JsValue::from_str(&format!("USER: {user} ")));
        let rounds_participated =
            self.users_round_history.get(&user).cloned().unwrap_or_else(Vec::new);
        if rounds_participated.len().eq(&0) {
            return 0.0;
        }
        // `current_round - ROUND_IMPORTANCE_DECAY_OFFSET` below is u32 subtraction. With overflow
        // checks off (the release wasm build) a `current_round` below the offset would silently
        // wrap to a huge value instead of erroring; guard it so a round reset fails loudly.
        assert!(
            self.current_round >= ROUND_IMPORTANCE_DECAY_OFFSET,
            "history bonus offset is bigger than current round"
        );
        // calculate weights sum
        let mut rounds_weights_sum = 0.0;
        for round in rounds_participated {
            // console::log_1(&JsValue::from_str(&format!("ROUND: {round} /rounds_participated")));
            let x_offset: f64 = (self.current_round - ROUND_IMPORTANCE_DECAY_OFFSET) as f64;
            let round_weight: f64 =
                generalised_logistic_function(0.0, 1.0, 1.0, 1.0, 1.0, 4.0, x_offset, round as f64);
            // console::log_1(&JsValue::from_str(&format!("weight {round_weight}")));
            if round < ACTIVE_VOTES_HISTORY_OLDEST_ROUND {
                rounds_weights_sum += round_weight;
                // console::log_1(&JsValue::from_str(&format!("PRE 32")));
            } else {
                // get votes from given round
                match self.votes_per_round.get(&round) {
                    Some(votes) => {
                        // multiply weight by ratio of active votes in given round
                        let with_ratio = round_weight * calculate_active_votes_ratio(&user, votes);
                        rounds_weights_sum += with_ratio;
                        // console::log_1(&JsValue::from_str(&format!(
                        //     "raw: {round_weight} with ratio: {with_ratio}"
                        // )));
                    }
                    None => {
                        // console::log_1(&JsValue::from_str(&format!(
                        //     "missing votes for {user} from this {round} round"
                        // )));
                    }
                }
            }
        }
        // pass the value into logistic curve
        generalised_logistic_function(0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 5.0, rounds_weights_sum)
    }
}

impl Neuron for PriorVotingHistoryNeuron {
    fn name(&self) -> String {
        "prior_voting_history_neuron".to_string()
    }

    fn calculate_result(&self, users: &[String]) -> HashMap<String, f64> {
        let mut result = HashMap::new();
        for user in users {
            let bonus = self.calculate_bonus(user.to_string());
            result.insert(user.into(), bonus);
        }
        result
    }
}

fn calculate_active_votes_ratio(user: &str, votes: &HashMap<String, HashMap<String, Vote>>) -> f64 {
    let mut total_votes_count: f64 = 0.0;
    let mut active_votes_count: f64 = 0.0;
    // iterate over all submissions
    votes.into_iter().for_each(|(_submission_name, votes)| {
        // get users vote for this submission
        match votes.get(user) {
            Some(vote) => match vote {
                Vote::Yes | Vote::No => active_votes_count += 1.0,
                Vote::Abstain | Vote::Delegate => {}
            },
            None => {
                //     console::log_1(&JsValue::from_str(&format!(
                //     "user {user} missing vote for submission {submission_name}"
                // )))
            }
        }
        total_votes_count += 1.0;
    });
    (active_votes_count / total_votes_count).max(ACTIVE_VOTES_MIN_RATIO)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_votes_ratio() {
        let submission1_votes: HashMap<String, Vote> =
            HashMap::from([("user1".to_string(), Vote::Yes)]);
        let submission2_votes: HashMap<String, Vote> =
            HashMap::from([("user1".to_string(), Vote::Yes)]);
        let submission3_votes: HashMap<String, Vote> =
            HashMap::from([("user1".to_string(), Vote::No)]);
        let submission4_votes: HashMap<String, Vote> =
            HashMap::from([("user1".to_string(), Vote::Delegate)]);
        let submission5_votes: HashMap<String, Vote> =
            HashMap::from([("user1".to_string(), Vote::Delegate)]);
        let votes: HashMap<String, HashMap<String, Vote>> = HashMap::from([
            ("submission1".to_string(), submission1_votes),
            ("submission2".to_string(), submission2_votes),
            ("submission3".to_string(), submission3_votes),
            ("submission4".to_string(), submission4_votes),
            ("submission5".to_string(), submission5_votes),
        ]);
        let result = calculate_active_votes_ratio("user1", &votes);
        assert_eq!(result, 0.6)
    }
    #[test]
    fn active_votes_ratio_no_less_than_cap() {
        let submission1_votes: HashMap<String, Vote> =
            HashMap::from([("user1".to_string(), Vote::Yes)]);
        let submission2_votes: HashMap<String, Vote> =
            HashMap::from([("user1".to_string(), Vote::Delegate)]);
        let submission3_votes: HashMap<String, Vote> =
            HashMap::from([("user1".to_string(), Vote::Delegate)]);
        let submission4_votes: HashMap<String, Vote> =
            HashMap::from([("user1".to_string(), Vote::Delegate)]);
        let submission5_votes: HashMap<String, Vote> =
            HashMap::from([("user1".to_string(), Vote::Delegate)]);
        let votes: HashMap<String, HashMap<String, Vote>> = HashMap::from([
            ("submission1".to_string(), submission1_votes),
            ("submission2".to_string(), submission2_votes),
            ("submission3".to_string(), submission3_votes),
            ("submission4".to_string(), submission4_votes),
            ("submission5".to_string(), submission5_votes),
        ]);
        let result = calculate_active_votes_ratio("user1", &votes);
        assert_eq!(result, ACTIVE_VOTES_MIN_RATIO)
    }

    // ---- Behavioral scenario tests for `calculate_bonus` / `calculate_result` ----
    //
    // Expected values are re-derived through the same `generalised_logistic_function`
    // calls the production code uses (see `round_weight` / `final_squash` below), so the
    // assertions document the math instead of pasting opaque floats.
    //
    // Round-number note: `round < 32` is a data-availability cutoff. Rounds < 32 take the
    // raw-weight path; rounds >= 32 take the active-ratio path and contribute 0 unless a
    // matching `votes_per_round` entry exists. The weight/recency/participation tests below
    // therefore use rounds < 32 with `current_round = 38`.

    const EPS: f64 = 1e-9;

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < EPS, "expected {expected}, got {actual}");
    }

    // Per-round importance weight: logistic(0,1,1,1,1,4, current_round-8) applied to `round`.
    fn round_weight(current_round: f64, round: f64) -> f64 {
        generalised_logistic_function(0.0, 1.0, 1.0, 1.0, 1.0, 4.0, current_round - 8.0, round)
    }

    // Final squash applied to the summed weights: logistic(0,1,1,1,1,1,5).
    fn final_squash(sum: f64) -> f64 {
        generalised_logistic_function(0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 5.0, sum)
    }

    fn history(entries: &[(&str, &[u32])]) -> HashMap<String, Vec<u32>> {
        entries.iter().map(|(u, rounds)| ((*u).to_string(), rounds.to_vec())).collect()
    }

    // Build a `votes_per_round` map: one round, one user, N submissions each with the given vote.
    fn votes_for_round(
        round: u32,
        user: &str,
        per_submission: &[Vote],
    ) -> HashMap<u32, HashMap<String, HashMap<String, Vote>>> {
        let mut submissions: HashMap<String, HashMap<String, Vote>> = HashMap::new();
        for (i, vote) in per_submission.iter().enumerate() {
            let mut sub = HashMap::new();
            sub.insert(user.to_string(), vote.clone());
            submissions.insert(format!("submission{i}"), sub);
        }
        HashMap::from([(round, submissions)])
    }

    #[test]
    fn empty_history_returns_zero() {
        let neuron =
            PriorVotingHistoryNeuron::from_data(history(&[("emptyvec", &[])]), HashMap::new(), 38);
        // user absent from the map entirely
        assert_close(neuron.calculate_bonus("ghost".to_string()), 0.0);
        // user present but with no rounds
        assert_close(neuron.calculate_bonus("emptyvec".to_string()), 0.0);
    }

    #[test]
    fn single_recent_round_bonus() {
        // round 31 < 32 -> raw-weight path; current_round 38 keeps `current_round - 8` safe.
        let neuron =
            PriorVotingHistoryNeuron::from_data(history(&[("alice", &[31])]), HashMap::new(), 38);
        let expected = final_squash(round_weight(38.0, 31.0));
        assert_close(neuron.calculate_bonus("alice".to_string()), expected);
    }

    #[test]
    fn more_recent_rounds_beat_older_rounds() {
        // same count (1), different recency; the round weight is monotonic in the round number.
        let neuron = PriorVotingHistoryNeuron::from_data(
            history(&[("older", &[29]), ("newer", &[31])]),
            HashMap::new(),
            38,
        );
        let older = neuron.calculate_bonus("older".to_string());
        let newer = neuron.calculate_bonus("newer".to_string());
        assert!(newer > older, "more recent ({newer}) should exceed older ({older})");
    }

    #[test]
    fn bonus_increases_with_participation_count() {
        // "voted in 3 recent rounds -> bigger bonus": A in [29], B in [29,30,31], all < 32.
        let neuron = PriorVotingHistoryNeuron::from_data(
            history(&[("a", &[29]), ("b", &[29, 30, 31])]),
            HashMap::new(),
            38,
        );
        let a = neuron.calculate_bonus("a".to_string());
        let b = neuron.calculate_bonus("b".to_string());
        let expected_b = final_squash(
            round_weight(38.0, 29.0) + round_weight(38.0, 30.0) + round_weight(38.0, 31.0),
        );
        assert_close(b, expected_b);
        assert!(b > a, "3-round user ({b}) should exceed 1-round user ({a})");
    }

    #[test]
    fn post_32_round_scales_by_active_ratio() {
        // round 35 >= 32 with a votes entry: 3 active (Yes/No) of 4 submissions -> ratio 0.75.
        let votes = votes_for_round(35, "alice", &[Vote::Yes, Vote::Yes, Vote::No, Vote::Delegate]);
        let neuron = PriorVotingHistoryNeuron::from_data(history(&[("alice", &[35])]), votes, 38);
        let expected = final_squash(round_weight(38.0, 35.0) * 0.75);
        assert_close(neuron.calculate_bonus("alice".to_string()), expected);
    }

    #[test]
    fn post_32_round_without_votes_contributes_nothing() {
        // same round 35 but no votes_per_round entry -> the round adds nothing; sum stays 0.
        let neuron =
            PriorVotingHistoryNeuron::from_data(history(&[("alice", &[35])]), HashMap::new(), 38);
        assert_close(neuron.calculate_bonus("alice".to_string()), final_squash(0.0));
    }

    #[test]
    fn active_ratio_floor_applies_in_bonus() {
        // 1 active of 5 -> true ratio 0.2, floored to ACTIVE_VOTES_MIN_RATIO (0.5).
        let votes = votes_for_round(
            35,
            "alice",
            &[
                Vote::Yes,
                Vote::Delegate,
                Vote::Delegate,
                Vote::Delegate,
                Vote::Delegate,
            ],
        );
        let neuron = PriorVotingHistoryNeuron::from_data(history(&[("alice", &[35])]), votes, 38);
        let expected = final_squash(round_weight(38.0, 35.0) * ACTIVE_VOTES_MIN_RATIO);
        assert_close(neuron.calculate_bonus("alice".to_string()), expected);
    }

    #[test]
    fn calculate_result_maps_all_users() {
        let neuron = PriorVotingHistoryNeuron::from_data(
            history(&[("a", &[29]), ("b", &[30, 31])]),
            HashMap::new(),
            38,
        );
        let users = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let result = neuron.calculate_result(&users);
        assert_eq!(result.len(), 3);
        for u in &users {
            assert_close(*result.get(u).unwrap(), neuron.calculate_bonus(u.clone()));
        }
    }

    #[test]
    #[should_panic(expected = "history bonus offset is bigger than current round")]
    fn current_round_below_offset_panics() {
        // A current_round below ROUND_IMPORTANCE_DECAY_OFFSET (e.g. after a round reset to 0) would
        // underflow the u32 offset subtraction and silently wrap in the release wasm build, so
        // calculate_bonus guards it with an explicit assert. Holds in both debug and release.
        let neuron =
            PriorVotingHistoryNeuron::from_data(history(&[("alice", &[3])]), HashMap::new(), 5);
        let _ = neuron.calculate_bonus("alice".to_string());
    }

    #[test]
    fn active_votes_ratio_empty_submissions() {
        // NOTE: empty submission map -> 0.0/0.0 = NaN, then NaN.max(0.5) returns 0.5
        // (f64::max ignores NaN). Characterizing current behavior.
        let votes: HashMap<String, HashMap<String, Vote>> = HashMap::new();
        assert_close(calculate_active_votes_ratio("user1", &votes), 0.5);
    }
}
