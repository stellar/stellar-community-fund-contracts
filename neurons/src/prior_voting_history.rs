use crate::neurons::Neuron;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug)]
pub struct PriorVotingHistoryNeuron {
    users_round_history: HashMap<String, Vec<u32>>,
}

impl PriorVotingHistoryNeuron {
    pub fn from_data(users_round_history: HashMap<String, Vec<u32>>) -> Self {
        Self {
            users_round_history,
        }
    }
}

fn generalised_logistic_function(
    a: f64,
    k: f64,
    c: f64,
    q: f64,
    b: f64,
    nu: f64,
    x_off: f64,
    x: f64,
) -> f64 {
    a + (k - a) / (f64::powf(c + q * f64::exp(-b * (x - x_off)), 1.0 / nu))
}

fn round_weight(round: u32) -> f64 {
    generalised_logistic_function(0.0, 1.0, 1.0, 1.0, 1.0, 4.0, 22.0, round as f64)
}

fn bonus(rounds_weights_sum: f64) -> f64 {
    generalised_logistic_function(0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 5.0, rounds_weights_sum)
}

fn calculate_bonus(rounds_participated: &[u32]) -> f64 {
    let rounds_weights_sum = rounds_participated
        .iter()
        .map(|round| round_weight(*round))
        .sum();
    bonus(rounds_weights_sum)
}

impl Neuron for PriorVotingHistoryNeuron {
    fn name(&self) -> String {
        "prior_voting_history_neuron".to_string()
    }

    fn calculate_result(&self, users: &[String]) -> HashMap<String, f64> {
        let mut result = HashMap::new();

        for user in users {
            let bonus = calculate_bonus(
                &self
                    .users_round_history
                    .get(user)
                    .cloned()
                    .unwrap_or_else(Vec::new),
            );
            result.insert(user.into(), bonus);
        }

        result
    }
}
