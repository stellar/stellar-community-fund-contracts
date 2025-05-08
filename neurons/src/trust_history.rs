use crate::neurons::Neuron;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

#[derive(Clone, Debug)]
pub struct TrustHistoryNeuron {
    round: usize,
}

impl TrustHistoryNeuron {
    pub fn new(round: usize) -> Self {
        Self { round }
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

impl Neuron for TrustHistoryNeuron {
    fn name(&self) -> String {
        format!("trust_history_neuron")
    }

    fn calculate_result(&self, _users: &[String]) -> HashMap<String, f64> {
        let mut users_trust_history: HashMap<String, Vec<f64>> = HashMap::new();

        for i in self.round - 1..=self.round {
            println!("Reading round file: {}", i);
            let path = format!("result/trust_graph_neuron_{}.json", i);
            let file = File::open(path).unwrap();
            let reader = BufReader::new(file);

            let user_trust: HashMap<String, f64> = serde_json::from_reader(reader).unwrap();

            user_trust
                .iter()
                .for_each(|(user, trust)| match users_trust_history.get_mut(user) {
                    Some(trust_vec) => {
                        trust_vec.push(*trust);
                    }
                    None => {
                        let _ = users_trust_history.insert(user.to_string(), vec![*trust]);
                    }
                });
        }
        let mut result = HashMap::new();

        // calculate diff in % of every user beetween last and current round
        users_trust_history.iter().for_each(|(user, trust_vec)| {
            let length = trust_vec.len();
            let current_trust = trust_vec[length - 1];
            let previous_trust = trust_vec[length - 2];
            let diff_percent = (current_trust / previous_trust) * 100.0;
            // NaN - previous == 0 && current == 0
            // inf - previous == 0 && current != 0

            if diff_percent.is_nan() {
                result.insert(user.into(), 0.0);
            } else if diff_percent.is_infinite() {
                result.insert(user.into(), current_trust);
            } else {
                let log_fn_out = generalised_logistic_function(
                    30.0,
                    100.0,
                    1.0,
                    1.0,
                    0.2,
                    3.0,
                    70.0,
                    diff_percent,
                );
                let outcome = (log_fn_out * current_trust) / 100.0;
                result.insert(user.into(), outcome);
            }
        });

        result
    }
}
