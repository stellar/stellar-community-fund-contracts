use crate::{neurons::Neuron, types::generalised_logistic_function};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct TrustHistoryNeuron {
    round: usize,
    trust_graph_neurons_results: HashMap<String, HashMap<String, f64>>,
}

impl TrustHistoryNeuron {
    pub fn from_data(
        round: usize,
        trust_graph_neurons_results: HashMap<String, HashMap<String, f64>>,
    ) -> Self {
        Self {
            round,
            trust_graph_neurons_results,
        }
    }
}

impl Neuron for TrustHistoryNeuron {
    fn name(&self) -> String {
        format!("trust_history_neuron")
    }

    fn calculate_result(&self, _users: &[String]) -> HashMap<String, f64> {
        let mut users_trust_history: HashMap<String, Vec<f64>> = HashMap::new();

        for i in self.round - 1..=self.round {
            let user_trust: HashMap<String, f64> = self
                .trust_graph_neurons_results
                .get(&format!("trust_graph_neuron_{}", i))
                .unwrap()
                .clone();

            user_trust.iter().for_each(|(user, trust)| match users_trust_history.get_mut(user) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trust_graph::TrustGraphNeuron;

    const EPS: f64 = 1e-9;

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < EPS, "expected {expected}, got {actual}");
    }

    // The exact logistic the neuron applies to `diff_percent` before scaling by current_trust.
    fn trust_logistic(diff_percent: f64) -> f64 {
        generalised_logistic_function(30.0, 100.0, 1.0, 1.0, 0.2, 3.0, 70.0, diff_percent)
    }

    // outcome = logistic(diff_percent) * current_trust / 100.0, where diff_percent = current/prev*100.
    fn expected_outcome(previous: f64, current: f64) -> f64 {
        trust_logistic(current / previous * 100.0) * current / 100.0
    }

    // Build the results map with the two round keys the neuron reads for `round`.
    fn results(
        round: usize,
        previous: &[(&str, f64)],
        current: &[(&str, f64)],
    ) -> HashMap<String, HashMap<String, f64>> {
        let to_map = |entries: &[(&str, f64)]| -> HashMap<String, f64> {
            entries.iter().map(|(u, t)| ((*u).to_string(), *t)).collect()
        };
        let mut map = HashMap::new();
        map.insert(format!("trust_graph_neuron_{}", round - 1), to_map(previous));
        map.insert(format!("trust_graph_neuron_{round}"), to_map(current));
        map
    }

    #[test]
    fn trust_unchanged() {
        let neuron = TrustHistoryNeuron::from_data(30, results(30, &[("alice", 0.8)], &[("alice", 0.8)]));
        let out = neuron.calculate_result(&[]);
        assert_close(*out.get("alice").unwrap(), trust_logistic(100.0) * 0.8 / 100.0);
    }

    #[test]
    fn trust_increased_gives_higher_outcome() {
        let neuron = TrustHistoryNeuron::from_data(30, results(30, &[("alice", 0.5)], &[("alice", 0.8)]));
        let out = neuron.calculate_result(&[]);
        let got = *out.get("alice").unwrap();
        assert_close(got, expected_outcome(0.5, 0.8));
        // gaining trust beats staying flat at the same current value
        assert!(got > trust_logistic(100.0) * 0.8 / 100.0);
    }

    #[test]
    fn trust_decreased_applies_penalty() {
        let neuron = TrustHistoryNeuron::from_data(30, results(30, &[("alice", 0.8)], &[("alice", 0.5)]));
        let out = neuron.calculate_result(&[]);
        let got = *out.get("alice").unwrap();
        assert_close(got, expected_outcome(0.8, 0.5));
        // losing trust is strictly worse than staying flat at the same current value
        assert!(got < trust_logistic(100.0) * 0.5 / 100.0);
    }

    #[test]
    fn severe_trust_loss() {
        let neuron = TrustHistoryNeuron::from_data(30, results(30, &[("alice", 1.0)], &[("alice", 0.1)]));
        let out = neuron.calculate_result(&[]);
        assert_close(*out.get("alice").unwrap(), expected_outcome(1.0, 0.1));
        // diff_percent = 10 is far left of the curve midpoint (70) -> near the floor (a = 30)
        assert!(trust_logistic(10.0) < 35.0);
    }

    #[test]
    fn previous_zero_current_zero_returns_zero() {
        let neuron = TrustHistoryNeuron::from_data(30, results(30, &[("alice", 0.0)], &[("alice", 0.0)]));
        let out = neuron.calculate_result(&[]);
        // diff_percent is NaN -> the NaN branch returns 0.0
        assert_close(*out.get("alice").unwrap(), 0.0);
    }

    #[test]
    fn previous_zero_current_positive_returns_current() {
        let neuron = TrustHistoryNeuron::from_data(30, results(30, &[("alice", 0.0)], &[("alice", 0.7)]));
        let out = neuron.calculate_result(&[]);
        // diff_percent is +inf -> the infinite branch returns current_trust as-is
        assert_close(*out.get("alice").unwrap(), 0.7);
    }

    #[test]
    fn current_zero_previous_positive_is_zero() {
        let neuron = TrustHistoryNeuron::from_data(30, results(30, &[("alice", 0.6)], &[("alice", 0.0)]));
        let out = neuron.calculate_result(&[]);
        // diff_percent = 0 (finite) -> else branch -> logistic(...,0) * 0 / 100 == 0
        assert_close(*out.get("alice").unwrap(), 0.0);
    }

    #[test]
    fn multiple_users_independent() {
        let neuron = TrustHistoryNeuron::from_data(
            30,
            results(
                30,
                &[("up", 0.4), ("down", 0.9), ("flat", 0.5)],
                &[("up", 0.8), ("down", 0.3), ("flat", 0.5)],
            ),
        );
        let out = neuron.calculate_result(&[]);
        assert_close(*out.get("up").unwrap(), expected_outcome(0.4, 0.8));
        assert_close(*out.get("down").unwrap(), expected_outcome(0.9, 0.3));
        assert_close(*out.get("flat").unwrap(), expected_outcome(0.5, 0.5));
    }

    // ---- Headline scenario: "trusted by N users -> trust-history bonus" ----
    //
    // Wires trust_graph (per-round PageRank from trust edges) into trust_history exactly as
    // lib.rs does, but inline. A target user is trusted by 3 users in the previous round and 10
    // in the current round. A "celebrity" trusted by everyone (plus two extra trusters) anchors
    // the PageRank maximum so the target stays interior (neither global min nor max), which keeps
    // it on the clean logistic branch in both rounds. PageRank values are normalized and graph-
    // dependent, so we assert ordering/monotonicity rather than a hardcoded float.
    #[test]
    fn combined_more_trusters_raises_trust_history_bonus() {
        let target = "target".to_string();

        let build_edges = |trusters: &[&str]| -> HashMap<String, Vec<String>> {
            let mut edges: HashMap<String, Vec<String>> = HashMap::new();
            // every truster trusts both the celebrity and the target
            for t in trusters {
                edges.insert((*t).to_string(), vec!["celeb".to_string(), target.clone()]);
            }
            // two extra users trust only the celebrity, keeping celeb strictly above target
            edges.insert("x1".to_string(), vec!["celeb".to_string()]);
            edges.insert("x2".to_string(), vec!["celeb".to_string()]);
            edges
        };

        let trusters_prev = ["t1", "t2", "t3"];
        let trusters_curr = ["t1", "t2", "t3", "t4", "t5", "t6", "t7", "t8", "t9", "t10"];

        // evaluate every node that appears in either round
        let users: Vec<String> = [
            "celeb", "target", "x1", "x2", "t1", "t2", "t3", "t4", "t5", "t6", "t7", "t8", "t9",
            "t10",
        ]
        .iter()
        .map(|s| (*s).to_string())
        .collect();

        let prev = TrustGraphNeuron::from_data(build_edges(&trusters_prev), 29).calculate_result(&users);
        let curr = TrustGraphNeuron::from_data(build_edges(&trusters_curr), 30).calculate_result(&users);

        // stage 1 sanity: the target gains normalized PageRank as more users trust it
        let prev_rank = *prev.get(&target).unwrap();
        let curr_rank = *curr.get(&target).unwrap();
        assert!(curr_rank > prev_rank, "target rank should grow with more trusters");

        let mut map = HashMap::new();
        map.insert("trust_graph_neuron_29".to_string(), prev);
        map.insert("trust_graph_neuron_30".to_string(), curr);

        let history_out = TrustHistoryNeuron::from_data(30, map).calculate_result(&[]);

        // stage 2: a gaining-trust trajectory (diff_percent > 100) beats the flat baseline
        let flat_baseline = trust_logistic(100.0) * curr_rank / 100.0;
        assert!(
            *history_out.get(&target).unwrap() > flat_baseline,
            "gaining-trust trajectory should beat the flat baseline"
        );
    }
}
