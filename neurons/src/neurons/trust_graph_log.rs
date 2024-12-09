use crate::neurons::Neuron;
use camino::Utf8Path;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;

#[derive(Clone, Debug)]
pub struct TrustGraphNeuronLog {
    trusted_for_user: HashMap<String, Vec<String>>,
}

impl TrustGraphNeuronLog {
    pub fn try_from_file(path: &Utf8Path) -> anyhow::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let trusted_for_user = serde_json::from_reader(reader)?;
        Ok(Self { trusted_for_user })
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

impl Neuron for TrustGraphNeuronLog {
    fn name(&self) -> String {
        "trust_graph_neuron_log".to_string()
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
