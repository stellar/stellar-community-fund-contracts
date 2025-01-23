use crate::neurons::Neuron;
use camino::Utf8Path;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, Write};

#[derive(Clone, Debug)]
pub struct TrustHistoryNeuron {
    start: usize,
    end: usize
}

impl TrustHistoryNeuron {
    pub fn new(rounds_range: (usize, usize)) -> Self {
        let (start, end) = rounds_range;
        Self { start, end }
    }
}
pub const DECIMALS: i64 = 1_000_000_000_000_000_000;

fn to_fixed_point_decimal(val: f64) -> i128 {
    (val * DECIMALS as f64) as i128
}
impl Neuron for TrustHistoryNeuron {
    fn name(&self) -> String {
        format!("trust_graph_neuron_log_{}_{}", self.start, self.end)
    }

    fn calculate_result(&self, users: &[String]) -> HashMap<String, f64> {
        let mut users_trust_history: HashMap<String, Vec<f64>> = HashMap::new();

        for i in self.start..self.end+1 {
            let path = format!("result/trust_graph_neuron_{}.json", i);
            let file = File::open(path).unwrap();
            let reader = BufReader::new(file);

            let user_trust: HashMap<String, f64> = serde_json::from_reader(reader).unwrap();

            user_trust.iter().for_each(|(user, trust)| {
                // let converted = to_fixed_point_decimal(*trust).to_string();
                match users_trust_history.get_mut(user) {
                    Some(trust_vec) => {
                        trust_vec.push(*trust);
                    },
                    None => {
                        let _ = users_trust_history.insert(user.to_string(), vec![*trust]);
                    }
                }
            });
        }

        // calculate diff in % of every user beetween last and current round
        let mut users_trust_recent_diff_percent: HashMap<String, Option<f64>> = HashMap::new();
        users_trust_history.iter().for_each(|(user, trust_vec)|{
            let length = trust_vec.len();
            let current_trust = trust_vec[length-1];
            let previous_trust = trust_vec[length-2];
            let diff_percent = (current_trust/previous_trust) * 100.0;
            // NaN - previous == 0 && current == 0
            // inf - previous == 0 && current != 0
            if diff_percent.is_nan() || diff_percent.is_infinite() {
                users_trust_recent_diff_percent.insert(user.to_string(), None);
            } else {
                users_trust_recent_diff_percent.insert(user.to_string(), Some(diff_percent));
            }
        });
        // println!("{:#?}", users_trust_recent_diff_percent);
        
        // do the calculation using logistic trust curve for every users diff,
        // take multiply current trust by outcome of above (current trust * % logistic negative bonus)

        let mut result = HashMap::new();
        for user in users {
            result.insert(user.into(), 0.0);
        }

        result

    }
}
// export data to json
// let json = serde_json::to_string(&users_trust_history).unwrap();
// let mut file = File::create("result/users_trust_history.json").unwrap();
// file.write_all(json.as_bytes()).unwrap();