use crate::neurons::Neuron;
use std::collections::{HashMap, HashSet};

// users with top X % highest trust score (equal or above) will be considered highly trusted
const HIGHLY_TRUSTED_PERCENT_THRESHOLD: usize = 10;
// users trusted by highly trusted users will get a bonus of X % of their own score
const HIGHLY_TRUSTED_PERCENT_BONUS: f64 = 15.0;

#[derive(Clone, Debug)]
pub struct TrustGraphNeuron {
    trusted_for_user: HashMap<String, Vec<String>>,
    round: u32,
}

impl TrustGraphNeuron {
    pub fn from_data(trusted_for_user: HashMap<String, Vec<String>>, round: u32) -> Self {
        Self {
            trusted_for_user,
            round,
        }
    }

    fn handle_page_rank(&self, users: &[String]) -> HashMap<String, f64> {
        let mut result: HashMap<String, f64> = HashMap::new();
        let mut nodes = HashSet::new();
        let mut edges = Vec::new();

        for (user, edge) in &self.trusted_for_user {
            nodes.insert(user.clone());
            for other_user in edge {
                nodes.insert(other_user.clone());
            }
            edges.push((user.clone(), edge.clone()));
        }
        let nodes: Vec<String> = nodes.into_iter().collect();

        let page_rank_result = calculate_page_rank(&nodes, &edges, 1000, 0.85);
        let page_rank_result = min_max_normalize_result(page_rank_result);

        for user in users {
            let page_rank = *page_rank_result.get(user).unwrap_or(&0.0);
            result.insert(user.into(), page_rank);
        }
        result
    }

    fn handle_highly_trusted_bonus(
        &self,
        trust_map: HashMap<String, f64>,
        percent_threshold: usize,
        percent_bonus: f64,
    ) -> HashMap<String, f64> {
        // ADDITIONAL BONUS if you're trusted by highly trusted user
        let mut result_with_bonus: HashMap<String, f64> = trust_map.clone();

        // calculate who has top X % highest trust
        let high_trust_value = calculate_high_trust_value(&trust_map, percent_threshold);

        // if you're trusted by someone whos trust score is higher than this, you get additional X % of your own score
        for (user, score) in &trust_map {
            // is user considered highly trusted
            if score >= &high_trust_value {
                // get all users he trusts
                // (someone can be trusted by a lot of users, but not trust anyone himself, in such case just skip)
                if let Some(trusted_for_this_user) = self.trusted_for_user.get(user) {
                    // give everyone a bonus
                    for u in trusted_for_this_user {
                        let res = result_with_bonus.get_mut(u).unwrap();
                        *res += (*res / 100.0) * percent_bonus;
                    }
                }
            }
        }

        // print only those results that have diff - for debug
        let mut with_bonus_count = 0;
        for (user, score) in &trust_map {
            let with_bonus = result_with_bonus.get(user).unwrap();
            if score != with_bonus {
                with_bonus_count += 1;
                println!("Score: {}, with bonus: {}", score, with_bonus);
            }
        }
        println!("{}/{}", with_bonus_count, trust_map.len());

        result_with_bonus
    }
}

impl Neuron for TrustGraphNeuron {
    fn name(&self) -> String {
        format!("trust_graph_neuron_{}", self.round)
    }
    fn calculate_result(&self, users: &[String]) -> HashMap<String, f64> {
        let page_rank_result = self.handle_page_rank(users);
        let highly_trusted_bonus_result = self.handle_highly_trusted_bonus(
            page_rank_result,
            HIGHLY_TRUSTED_PERCENT_THRESHOLD,
            HIGHLY_TRUSTED_PERCENT_BONUS,
        );
        // TODO maybe move here history bonus instead of json files
        highly_trusted_bonus_result
    }
}

#[allow(clippy::cast_precision_loss)]
fn calculate_page_rank(
    nodes: &Vec<String>,
    edges: &Vec<(String, Vec<String>)>,
    iterations: u32,
    damping_factor: f64,
) -> HashMap<String, f64> {
    let mut page_ranks: HashMap<String, f64> = HashMap::new();
    for node in nodes {
        page_ranks.insert(node.clone(), 1.0 / nodes.len() as f64);
    }
    for _ in 0..iterations {
        let mut new_ranks: HashMap<String, f64> = HashMap::new();
        for node in nodes {
            let mut rank = (1.0 - damping_factor) / nodes.len() as f64;
            for (other_node, other_node_edges) in edges {
                if other_node_edges.contains(node) {
                    let pr = page_ranks.get(other_node).unwrap_or(&0.0);
                    rank += (damping_factor * pr) / other_node_edges.len() as f64;
                }
            }
            new_ranks.insert(node.clone(), rank);
        }
        page_ranks = new_ranks;
    }

    page_ranks
}

fn min_max_normalize_result(result: HashMap<String, f64>) -> HashMap<String, f64> {
    let min = result.values().copied().reduce(f64::min).unwrap();
    let max = result.values().copied().reduce(f64::max).unwrap();

    result
        .into_iter()
        .map(|(key, value)| {
            let new_value = (value - min) / (max - min);
            (key, new_value)
        })
        .collect()
}

fn calculate_high_trust_value(trust_map: &HashMap<String, f64>, percent_threshold: usize) -> f64 {
    let mut trust_scores_sorted: Vec<f64> = trust_map.values().cloned().collect();
    trust_scores_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let target_index =
        trust_scores_sorted.len() - ((trust_scores_sorted.len() * percent_threshold) / 100).max(1);

    trust_scores_sorted.get(target_index).unwrap().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    macro_rules! assert_f64_near {
        ( $a:expr, $b:expr ) => {
            let eps = 0.001f64;
            assert!(
                ($a - $b).abs() < eps,
                "Values a = {}, b = {} are not near",
                $a,
                $b
            );
        };
    }

    #[test]
    fn calc_high_trust_value() {
        let mut trust_map: HashMap<String, f64> = HashMap::new();
        for x in 1..=100 {
            trust_map.insert(Uuid::new_v4().to_string(), x as f64);
        }
        assert_eq!(calculate_high_trust_value(&trust_map, 1), 100.0);
        assert_eq!(calculate_high_trust_value(&trust_map, 2), 99.0);
        assert_eq!(calculate_high_trust_value(&trust_map, 5), 96.0);
        assert_eq!(calculate_high_trust_value(&trust_map, 10), 91.0);
        assert_eq!(calculate_high_trust_value(&trust_map, 20), 81.0);
        assert_eq!(calculate_high_trust_value(&trust_map, 50), 51.0);
    }

    #[test]
    fn calc_highly_trusted_bonus() {
        let mut trusted_for_user = HashMap::new();
        trusted_for_user.insert("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        trusted_for_user.insert("B".to_string(), vec!["A".to_string()]);
        trusted_for_user.insert("C".to_string(), vec!["A".to_string(), "B".to_string()]);
        trusted_for_user.insert("D".to_string(), vec!["A".to_string()]);
        trusted_for_user.insert("E".to_string(), vec![]);

        let trust_graph_neuron = TrustGraphNeuron {
            trusted_for_user,
            round: 0,
        };

        let result = trust_graph_neuron.handle_page_rank(
            &["A", "B", "C", "D", "E"]
                .into_iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>(),
        );

        let with_bonus = trust_graph_neuron.handle_highly_trusted_bonus(result, 1, 100.0);

        assert_f64_near!(with_bonus.get("B").unwrap(), &1.408);
        assert_f64_near!(with_bonus.get("C").unwrap(), &0.931);
    }

    #[test]
    fn simple_page_rank() {
        let mut trusted_for_user = HashMap::new();
        trusted_for_user.insert("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        trusted_for_user.insert("B".to_string(), vec!["A".to_string()]);
        trusted_for_user.insert("C".to_string(), vec!["A".to_string(), "B".to_string()]);
        trusted_for_user.insert("D".to_string(), vec!["A".to_string()]);
        trusted_for_user.insert("E".to_string(), vec![]);

        let trust_graph_neuron = TrustGraphNeuron {
            trusted_for_user,
            round: 0,
        };

        let result = trust_graph_neuron.handle_page_rank(
            &["A", "B", "C", "D", "E"]
                .into_iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>(),
        );

        assert_f64_near!(result.get("A").unwrap(), &1.0);
        assert_f64_near!(result.get("B").unwrap(), &0.704);
        assert_f64_near!(result.get("C").unwrap(), &0.465);
        assert_f64_near!(result.get("D").unwrap(), &0.0);
        assert_f64_near!(result.get("E").unwrap(), &0.0);
    }
}
