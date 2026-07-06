use wasm_bindgen::JsValue;
use web_sys::console;

use crate::neurons::Neuron;
use std::collections::{HashMap, HashSet};

// users with top X % highest trust score (equal or above) will be considered highly trusted
const HIGHLY_TRUSTED_PERCENT_THRESHOLD: usize = 10;
// users trusted by highly trusted users will get a bonus of X % of their own score
const HIGHLY_TRUSTED_PERCENT_BONUS: f64 = 15.0;

#[derive(Clone, Debug)]
pub struct TrustGraphNeuron {
    trusted_for_user: HashMap<String, Vec<String>>,
}

impl TrustGraphNeuron {
    pub fn from_data(trusted_for_user: HashMap<String, Vec<String>>) -> Self {
        Self {
            trusted_for_user,
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
                        match result_with_bonus.get_mut(u) {
                            Some(res) => {
                                *res += (*res / 100.0) * percent_bonus;
                            }
                            None => {
                                console::log_1(&JsValue::from_str(&format!(
                                    "handle_highly_trusted_bonus missing: {u}"
                                )));
                            }
                        }
                    }
                }
            }
        }

        // print only those results that have diff - for debug
        // let mut with_bonus_count = 0;
        // for (user, score) in &trust_map {
        //     let with_bonus = result_with_bonus.get(user).unwrap();
        //     if score != with_bonus {
        //         with_bonus_count += 1;
        //         println!("Score: {}, with bonus: {}", score, with_bonus);
        //     }
        // }
        // println!("{}/{}", with_bonus_count, trust_map.len());

        result_with_bonus
    }
}

impl Neuron for TrustGraphNeuron {
    fn name(&self) -> String {
        format!("trust_graph_neuron")
    }
    fn calculate_result(&self, users: &[String]) -> HashMap<String, f64> {
        let page_rank_result = self.handle_page_rank(users);
        let highly_trusted_bonus_result = self.handle_highly_trusted_bonus(
            page_rank_result,
            HIGHLY_TRUSTED_PERCENT_THRESHOLD,
            HIGHLY_TRUSTED_PERCENT_BONUS,
        );
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

// Scale factor for min-max normalization output (0 to SCALE)
const NORMALIZATION_SCALE: f64 = 3.0;

fn min_max_normalize_result(result: HashMap<String, f64>) -> HashMap<String, f64> {
    let min = result.values().copied().reduce(f64::min).unwrap();
    let max = result.values().copied().reduce(f64::max).unwrap();

    result
        .into_iter()
        .map(|(key, value)| {
            let new_value = ((value - min) / (max - min)) * NORMALIZATION_SCALE;
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
            assert!(($a - $b).abs() < eps, "Values a = {}, b = {} are not near", $a, $b);
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
        };

        let result = trust_graph_neuron.handle_page_rank(
            &["A", "B", "C", "D", "E"]
                .into_iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>(),
        );

        let with_bonus = trust_graph_neuron.handle_highly_trusted_bonus(result, 1, 100.0);

        // PageRank normalized to 0-3, then 100% bonus applied
        assert_f64_near!(with_bonus.get("B").unwrap(), &4.226);
        assert_f64_near!(with_bonus.get("C").unwrap(), &2.794);
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
        };

        let result = trust_graph_neuron.handle_page_rank(
            &["A", "B", "C", "D", "E"]
                .into_iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>(),
        );

        // PageRank normalized to 0-3 range
        assert_f64_near!(result.get("A").unwrap(), &3.0);
        assert_f64_near!(result.get("B").unwrap(), &2.112);
        assert_f64_near!(result.get("C").unwrap(), &1.397);
        assert_f64_near!(result.get("D").unwrap(), &0.0);
        assert_f64_near!(result.get("E").unwrap(), &0.0);
    }

    fn users_vec(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn more_trusters_means_higher_pagerank() {
        // Single population so the node set (and the normalization base) is fixed: three targets
        // trusted by 1, 5 and 10 distinct users respectively. PageRank is relative, so this is the
        // honest framing of "trusted by more users -> higher score".
        let mut trusted_for_user: HashMap<String, Vec<String>> = HashMap::new();
        for (target, count) in [("target1", 1), ("target5", 5), ("target10", 10)] {
            for i in 0..count {
                trusted_for_user.insert(format!("{target}_truster{i}"), vec![target.to_string()]);
            }
        }
        let neuron = TrustGraphNeuron {
            trusted_for_user,
        };
        let ranks = neuron.handle_page_rank(&users_vec(&["target1", "target5", "target10"]));
        let r1 = ranks.get("target1").unwrap();
        let r5 = ranks.get("target5").unwrap();
        let r10 = ranks.get("target10").unwrap();
        assert!(r10 > r5, "10 trusters ({r10}) should beat 5 ({r5})");
        assert!(r5 > r1, "5 trusters ({r5}) should beat 1 ({r1})");
    }

    #[test]
    fn min_max_normalization_bounds() {
        // A clear hub: u1,u2,u3 all trust `hub`. After normalization the unique max is 3.0
        // and the (equal) trusters are the min at 0.0.
        let mut trusted_for_user: HashMap<String, Vec<String>> = HashMap::new();
        trusted_for_user.insert("u1".to_string(), vec!["hub".to_string()]);
        trusted_for_user.insert("u2".to_string(), vec!["hub".to_string()]);
        trusted_for_user.insert("u3".to_string(), vec!["hub".to_string()]);
        let neuron = TrustGraphNeuron {
            trusted_for_user,
        };
        let ranks = neuron.handle_page_rank(&users_vec(&["hub", "u1", "u2", "u3"]));
        assert_f64_near!(ranks.get("hub").unwrap(), &3.0);
        assert_f64_near!(ranks.get("u1").unwrap(), &0.0);
    }

    #[test]
    fn highly_trusted_bonus_threshold_boundary() {
        // 10 users, distinct scores 1..=10. At a 10% threshold only the single top-scoring user
        // counts as highly trusted: index = len - max(1, len*10/100) = 10 - 1 = 9 (the top score).
        let mut trust_map: HashMap<String, f64> = HashMap::new();
        for i in 1..=10 {
            trust_map.insert(format!("u{i}"), f64::from(i));
        }
        // top user u10 trusts u1,u2; u9 (NOT highly trusted at 10%) trusts u3.
        let mut trusted_for_user: HashMap<String, Vec<String>> = HashMap::new();
        trusted_for_user.insert("u10".to_string(), vec!["u1".to_string(), "u2".to_string()]);
        trusted_for_user.insert("u9".to_string(), vec!["u3".to_string()]);
        let neuron = TrustGraphNeuron {
            trusted_for_user,
        };

        let with_bonus = neuron.handle_highly_trusted_bonus(trust_map, 10, 15.0);

        // u1,u2 trusted by highly-trusted u10 -> +15% of their own score
        assert_f64_near!(with_bonus.get("u1").unwrap(), &(1.0 * 1.15));
        assert_f64_near!(with_bonus.get("u2").unwrap(), &(2.0 * 1.15));
        // u3 trusted only by u9 (below threshold) -> unchanged
        assert_f64_near!(with_bonus.get("u3").unwrap(), &3.0);
        // u10 itself untouched
        assert_f64_near!(with_bonus.get("u10").unwrap(), &10.0);
    }

    #[test]
    fn calculate_result_full_pipeline() {
        let mut trusted_for_user = HashMap::new();
        trusted_for_user.insert("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        trusted_for_user.insert("B".to_string(), vec!["A".to_string()]);
        trusted_for_user.insert("C".to_string(), vec!["A".to_string(), "B".to_string()]);
        trusted_for_user.insert("D".to_string(), vec!["A".to_string()]);
        trusted_for_user.insert("E".to_string(), vec![]);
        let neuron = TrustGraphNeuron {
            trusted_for_user,
        };
        let users = users_vec(&["A", "B", "C", "D", "E"]);

        let via_public = neuron.calculate_result(&users);
        let manual = neuron.handle_highly_trusted_bonus(
            neuron.handle_page_rank(&users),
            HIGHLY_TRUSTED_PERCENT_THRESHOLD,
            HIGHLY_TRUSTED_PERCENT_BONUS,
        );

        assert_eq!(via_public.len(), manual.len());
        for (k, v) in &manual {
            assert_f64_near!(via_public.get(k).unwrap(), v);
        }
    }

    #[test]
    fn name_returns_correct_value() {
        let neuron = TrustGraphNeuron {
            trusted_for_user: HashMap::new(),
        };
        assert_eq!(neuron.name(), "trust_graph_neuron");
    }
}