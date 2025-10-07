use crate::neurons::Neuron;
use crate::types::generalised_logistic_function;
use crate::Vote;
use serde::Deserialize;
use std::collections::HashMap;
use wasm_bindgen::JsValue;
use web_sys::{self, console};
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

    pub fn calculate_bonus(&self, user: String) -> f64 {
        // console::log_1(&JsValue::from_str(&format!("USER: {user} ")));
        let rounds_participated =
            self.users_round_history.get(&user).cloned().unwrap_or_else(Vec::new);
        if rounds_participated.len().eq(&0) {
            return 0.0;
        }
        // calculate weights sum
        let mut rounds_weights_sum = 0.0;
        for round in rounds_participated {
            // console::log_1(&JsValue::from_str(&format!("ROUND: {round} /rounds_participated")));
            let x_offset: f64 = (self.current_round - ROUND_IMPORTANCE_DECAY_OFFSET) as f64;
            let round_weight: f64 =
                // TODO MAKE X_OFF current round dependent
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
                        console::log_1(&JsValue::from_str(&format!(
                            "missing votes for {user} from this {round} round"
                        )));
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
    // let mut not_found: f64 = 0.0;
    // iterate over all submissions
    votes.into_iter().for_each(|(submission_name, votes)| {
        // get users vote for this submission
        match votes.get(user) {
            Some(vote) => match vote {
                Vote::Yes | Vote::No => active_votes_count += 1.0,
                Vote::Abstain | Vote::Delegate => {}
            },
            None => console::log_1(&JsValue::from_str(&format!(
                "user {user} missing vote for submission {submission_name}"
            ))),
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
}
