use crate::neurons::Neuron;
use std::collections::{HashMap, HashSet};

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

impl Neuron for TrustGraphNeuron {
    fn name(&self) -> String {
        format!("trust_graph_neuron_{}", self.round)
    }

    fn calculate_result(&self, users: &[String]) -> HashMap<String, f64> {
        let mut result = HashMap::new();

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
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn simple() {
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
        let result = trust_graph_neuron.calculate_result(
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
