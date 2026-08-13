use crate::{neurons::Neuron, types::generalised_logistic_function};
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use web_sys::console;

const HOW_DEEP: u32 = 32;
#[derive(Clone, Debug)]
pub struct TrustLossNeuron {
    round: u32,
    trusted_for_user_per_round: HashMap<u32, HashMap<String, Vec<String>>>,
    neurons_results_sum: HashMap<String, f64>,
}

impl TrustLossNeuron {
    pub fn from_data(round: u32, trusted_for_user_per_round: HashMap<u32, HashMap<String, Vec<String>>>, neurons_results_sum: HashMap<String, f64>) -> Self {
        Self {
            round,
            trusted_for_user_per_round,
            neurons_results_sum,
        }
    }
    fn find_most_recent_previous_trust_list(&self, target_user: &String) -> Option<Vec<String>> {
        let mut counter: u32 = self.round - 1;
        let mut most_recent_previous_trust_list: Option<Vec<String>> = None;

        while most_recent_previous_trust_list.is_none() && counter >= HOW_DEEP {
            if let Some(trusted_for_user) = self.trusted_for_user_per_round.get(&counter) {
                if let Some(trust_list) = trusted_for_user.get(target_user) {
                    most_recent_previous_trust_list = Some(trust_list.clone());
                }
            }
            counter -= 1;
        }
        most_recent_previous_trust_list
    }
    fn sum_nqg_of_users(&self, lost_trust: &Vec<String>) -> f64 {
        lost_trust.iter().fold(0.0, |acc, user| acc + self.neurons_results_sum.get(user).unwrap())
    }
}

impl Neuron for TrustLossNeuron {
    fn name(&self) -> String {
        format!("trust_loss_neuron")
    }

    fn calculate_result(&self, users: &[String]) -> HashMap<String, f64> {
        let mut result: HashMap<String, f64> = users.iter().map(|user| (user.to_string(), 0.0)).collect();
        let mut lost_trust_from: HashMap<String, Vec<String>> = HashMap::new();

        for user in users {
            // if this user doesn't have trust list for this round, skip this iteration completly
            // it implicates they couldn't have actively removed trust from someone.
            let current_trust_list = match self.trusted_for_user_per_round.get(&self.round) {
                Some(trusted_for_user_current_round) => match trusted_for_user_current_round.get(user) {
                    Some(current_trust_list) => current_trust_list.clone(),
                    None => continue,
                },
                None => continue,
            };
            // if this user doesn't have trust list from any previous round its a fresh account, in which case skip
            // but if this user has a trust list in any previous round, use it as a reference, return only if none found
            let previous_trust_list = match self.find_most_recent_previous_trust_list(user) {
                Some(previous_trust_list) => previous_trust_list,
                None => continue,
            };
            // iterate over who this user trusted in previous round
            previous_trust_list.iter().for_each(|trusted_user| {
                if !current_trust_list.contains(trusted_user) {
                    // for this trusted_user insert a record that he was untrusted by this user
                    let untrusting_users_list: &mut Vec<String> = lost_trust_from.entry(trusted_user.to_string()).or_insert(vec![]);
                    untrusting_users_list.push(user.to_string());
                }
            });
        }

        // calculate a sum of nqg scores of all users who untrusted each user
        let untrusted_by_nqg_amount: HashMap<String, f64> = lost_trust_from
            .iter()
            .map(|(user, lost_trust)| (user.to_string(), self.sum_nqg_of_users(lost_trust)))
            .collect();

        #[cfg(target_arch = "wasm32")]
        console::log_1(&JsValue::from_str(&format!("untrusted_by_nqg_amount: {:?}", untrusted_by_nqg_amount)));

        // pass those sums through logistic curve
        for (affected_user, amount) in untrusted_by_nqg_amount {
            let percentage_of_nqg_to_remove = generalised_logistic_function(0.0, 100.0, 1.0, 1.0, 0.2, 1.0, 50.0, amount);
            if let Some(value) = result.get_mut(&affected_user) {
                *value = -self.neurons_results_sum.get(&affected_user).unwrap() * percentage_of_nqg_to_remove / 100.0;
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::generalised_logistic_function;

    const EPS: f64 = 1e-9;

    fn users(names: &[&str]) -> Vec<String> {
        names.iter().map(|x| x.to_string()).collect()
    }

    fn trust_list(names: &[&str]) -> Vec<String> {
        names.iter().map(|x| x.to_string()).collect()
    }

    fn nqg_for(name: &str) -> f64 {
        match name {
            "alice" => 10.0,
            "bob" => 15.0,
            "charlie" => 12.0,
            "dave" => 8.0,
            "eve" => 20.0,
            "tom" => 18.0,
            "andy" => 25.0,
            "john" => 5.0,
            "adam" => 3.0,
            "newuser" => 7.0,
            "unknown_user" => 2.0,
            _ => 1.0,
        }
    }

    fn nqg_map(names: &[&str]) -> HashMap<String, f64> {
        names.iter().map(|n| (n.to_string(), nqg_for(n))).collect()
    }

    fn expected_loss(untrusting_nqg_sum: f64, affected_user_nqg: f64) -> f64 {
        let pct = generalised_logistic_function(0.0, 100.0, 1.0, 1.0, 0.2, 1.0, 50.0, untrusting_nqg_sum);
        -affected_user_nqg * pct / 100.0
    }

    fn assert_result_close(result: &HashMap<String, f64>, user_list: &[&str], overrides: &[(&str, f64)]) {
        assert_eq!(result.len(), user_list.len());
        for name in user_list {
            assert!(result.contains_key(*name), "missing key {name}");
        }
        for (name, expected) in overrides {
            let actual = result[*name];
            assert!(
                (actual - expected).abs() < EPS,
                "for {name}: expected {expected}, got {actual}"
            );
        }
        for name in user_list {
            if !overrides.iter().any(|(n, _)| n == name) {
                assert!(
                    result[*name].abs() < EPS,
                    "expected {name} to be 0.0, got {}",
                    result[*name]
                );
            }
        }
    }

    #[test]
    fn no_loss_no_gain() {
        let mut trusted_for_user_per_round: HashMap<u32, HashMap<String, Vec<String>>> = HashMap::new();

        let mut trust_44: HashMap<String, Vec<String>> = HashMap::new();
        trust_44.insert("alice".to_string(), trust_list(&["tom", "bob", "andy", "john"]));
        trust_44.insert("tom".to_string(), trust_list(&["alice", "bob", "andy", "john"]));
        trust_44.insert("bob".to_string(), trust_list(&["tom", "alice", "andy", "john"]));
        trust_44.insert("andy".to_string(), trust_list(&["tom", "bob", "alice", "john"]));
        trust_44.insert("john".to_string(), trust_list(&["tom", "bob", "alice", "andy"]));
        trusted_for_user_per_round.insert(44, trust_44);

        let mut trust_43: HashMap<String, Vec<String>> = HashMap::new();
        trust_43.insert("alice".to_string(), trust_list(&["tom", "bob", "andy", "john"]));
        trust_43.insert("tom".to_string(), trust_list(&["alice", "bob", "andy", "john"]));
        trust_43.insert("bob".to_string(), trust_list(&["tom", "alice", "andy", "john"]));
        trust_43.insert("andy".to_string(), trust_list(&["tom", "bob", "alice", "john"]));
        trust_43.insert("john".to_string(), trust_list(&["tom", "bob", "alice", "andy"]));
        trusted_for_user_per_round.insert(43, trust_43);

        let user_list = &["alice", "tom", "bob", "andy", "john", "adam"];
        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round, nqg_map(user_list));

        assert_result_close(&neuron.calculate_result(&users(user_list)), user_list, &[]);
    }

    #[test]
    fn gain() {
        let mut trusted_for_user_per_round: HashMap<u32, HashMap<String, Vec<String>>> = HashMap::new();

        let mut trust_44: HashMap<String, Vec<String>> = HashMap::new();
        trust_44.insert("alice".to_string(), trust_list(&["tom", "bob", "andy", "john", "adam"]));
        trust_44.insert("tom".to_string(), trust_list(&["alice", "bob", "andy", "john"]));
        trust_44.insert("bob".to_string(), trust_list(&["tom", "alice", "andy", "john", "adam"]));
        trust_44.insert("andy".to_string(), trust_list(&["tom", "bob", "alice", "john"]));
        trust_44.insert("john".to_string(), trust_list(&["tom", "bob", "alice", "andy", "adam"]));
        trusted_for_user_per_round.insert(44, trust_44);

        let mut trust_43: HashMap<String, Vec<String>> = HashMap::new();
        trust_43.insert("alice".to_string(), trust_list(&["tom", "bob", "andy", "john"]));
        trust_43.insert("tom".to_string(), trust_list(&["alice", "bob", "andy", "john"]));
        trust_43.insert("bob".to_string(), trust_list(&["tom", "alice", "andy", "john"]));
        trust_43.insert("andy".to_string(), trust_list(&["tom", "bob", "alice", "john"]));
        trust_43.insert("john".to_string(), trust_list(&["tom", "bob", "alice", "andy"]));
        trusted_for_user_per_round.insert(43, trust_43);

        let user_list = &["alice", "tom", "bob", "andy", "john", "adam"];
        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round, nqg_map(user_list));

        assert_result_close(&neuron.calculate_result(&users(user_list)), user_list, &[]);
    }

    #[test]
    fn loss() {
        let mut trusted_for_user_per_round: HashMap<u32, HashMap<String, Vec<String>>> = HashMap::new();

        let mut trust_44: HashMap<String, Vec<String>> = HashMap::new();
        trust_44.insert("alice".to_string(), trust_list(&["tom", "andy", "john", "adam"]));
        trust_44.insert("tom".to_string(), trust_list(&["alice", "bob", "john"]));
        trust_44.insert("bob".to_string(), trust_list(&["tom", "alice", "john", "adam"]));
        trust_44.insert("andy".to_string(), trust_list(&["tom", "bob", "alice", "john"]));
        trust_44.insert("john".to_string(), trust_list(&["tom", "bob", "alice", "andy", "adam"]));
        trusted_for_user_per_round.insert(44, trust_44);

        let mut trust_43: HashMap<String, Vec<String>> = HashMap::new();
        trust_43.insert("alice".to_string(), trust_list(&["tom", "bob", "andy", "john"]));
        trust_43.insert("tom".to_string(), trust_list(&["alice", "bob", "andy", "john"]));
        trust_43.insert("bob".to_string(), trust_list(&["tom", "alice", "andy", "john"]));
        trust_43.insert("andy".to_string(), trust_list(&["tom", "bob", "alice", "john"]));
        trust_43.insert("john".to_string(), trust_list(&["tom", "bob", "alice", "andy"]));
        trusted_for_user_per_round.insert(43, trust_43);

        let user_list = &["alice", "tom", "bob", "andy", "john"];
        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round, nqg_map(user_list));
        let result = neuron.calculate_result(&users(user_list));

        // alice (nqg=10) untrusted bob; tom (nqg=18) + bob (nqg=15) untrusted andy
        let expected_bob = expected_loss(nqg_for("alice"), nqg_for("bob"));
        let expected_andy = expected_loss(nqg_for("tom") + nqg_for("bob"), nqg_for("andy"));
        assert_result_close(&result, user_list, &[("bob", expected_bob), ("andy", expected_andy)]);
    }

    #[test]
    fn name_returns_correct_value() {
        let neuron = TrustLossNeuron::from_data(44, HashMap::new(), HashMap::new());
        assert_eq!(neuron.name(), "trust_loss_neuron");
    }

    #[test]
    fn empty_users_returns_empty_result() {
        let mut trusted_for_user_per_round: HashMap<u32, HashMap<String, Vec<String>>> = HashMap::new();
        let mut trust_44: HashMap<String, Vec<String>> = HashMap::new();
        trust_44.insert("alice".to_string(), trust_list(&["bob"]));
        trusted_for_user_per_round.insert(44, trust_44);

        let mut trust_43: HashMap<String, Vec<String>> = HashMap::new();
        trust_43.insert("alice".to_string(), trust_list(&["bob", "charlie"]));
        trusted_for_user_per_round.insert(43, trust_43);

        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round, HashMap::new());
        assert_eq!(neuron.calculate_result(&[]), HashMap::new());
    }

    #[test]
    fn empty_trust_data_returns_all_users_with_zero() {
        let user_list = &["alice", "bob"];
        let neuron = TrustLossNeuron::from_data(44, HashMap::new(), nqg_map(user_list));
        assert_result_close(&neuron.calculate_result(&users(user_list)), user_list, &[]);
    }

    #[test]
    fn user_without_current_round_trust_list_still_in_output() {
        let mut trusted_for_user_per_round: HashMap<u32, HashMap<String, Vec<String>>> = HashMap::new();

        let mut trust_44: HashMap<String, Vec<String>> = HashMap::new();
        trust_44.insert("alice".to_string(), trust_list(&["bob"]));
        trusted_for_user_per_round.insert(44, trust_44);

        let mut trust_43: HashMap<String, Vec<String>> = HashMap::new();
        trust_43.insert("alice".to_string(), trust_list(&["bob"]));
        trust_43.insert("charlie".to_string(), trust_list(&["bob", "alice"]));
        trusted_for_user_per_round.insert(43, trust_43);

        let user_list = &["alice", "bob", "charlie"];
        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round, nqg_map(user_list));
        let result = neuron.calculate_result(&users(user_list));
        assert_result_close(&result, user_list, &[]);
    }

    #[test]
    fn fresh_account_with_no_previous_trust_list_is_skipped() {
        let mut trusted_for_user_per_round: HashMap<u32, HashMap<String, Vec<String>>> = HashMap::new();

        let mut trust_44: HashMap<String, Vec<String>> = HashMap::new();
        trust_44.insert("alice".to_string(), trust_list(&["bob"]));
        trust_44.insert("newuser".to_string(), trust_list(&["alice"]));
        trusted_for_user_per_round.insert(44, trust_44);

        let mut trust_43: HashMap<String, Vec<String>> = HashMap::new();
        trust_43.insert("alice".to_string(), trust_list(&["bob", "charlie"]));
        trusted_for_user_per_round.insert(43, trust_43);

        let user_list = &["alice", "bob", "charlie", "newuser"];
        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round, nqg_map(user_list));
        let result = neuron.calculate_result(&users(user_list));

        // alice (nqg=10) untrusted charlie
        let expected_charlie = expected_loss(nqg_for("alice"), nqg_for("charlie"));
        assert_result_close(&result, user_list, &[("charlie", expected_charlie)]);
    }

    #[test]
    fn loss_not_in_users_list_is_not_recorded() {
        let mut trusted_for_user_per_round: HashMap<u32, HashMap<String, Vec<String>>> = HashMap::new();

        let mut trust_44: HashMap<String, Vec<String>> = HashMap::new();
        trust_44.insert("alice".to_string(), trust_list(&["bob"]));
        trusted_for_user_per_round.insert(44, trust_44);

        let mut trust_43: HashMap<String, Vec<String>> = HashMap::new();
        trust_43.insert("alice".to_string(), trust_list(&["bob", "outsider"]));
        trusted_for_user_per_round.insert(43, trust_43);

        let user_list = &["alice", "bob"];
        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round, nqg_map(user_list));
        let result = neuron.calculate_result(&users(user_list));
        assert_result_close(&result, user_list, &[]);
    }

    #[test]
    fn multiple_users_removing_trust_from_same_person_accumulates() {
        let mut trusted_for_user_per_round: HashMap<u32, HashMap<String, Vec<String>>> = HashMap::new();

        let mut trust_44: HashMap<String, Vec<String>> = HashMap::new();
        trust_44.insert("alice".to_string(), trust_list(&["bob"]));
        trust_44.insert("charlie".to_string(), trust_list(&["bob"]));
        trust_44.insert("dave".to_string(), trust_list(&["bob"]));
        trusted_for_user_per_round.insert(44, trust_44);

        let mut trust_43: HashMap<String, Vec<String>> = HashMap::new();
        trust_43.insert("alice".to_string(), trust_list(&["bob", "eve"]));
        trust_43.insert("charlie".to_string(), trust_list(&["bob", "eve"]));
        trust_43.insert("dave".to_string(), trust_list(&["bob", "eve"]));
        trusted_for_user_per_round.insert(43, trust_43);

        let user_list = &["alice", "bob", "charlie", "dave", "eve"];
        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round, nqg_map(user_list));
        let result = neuron.calculate_result(&users(user_list));

        // alice (10) + charlie (12) + dave (8) = 30 untrusted eve (20)
        let expected_eve = expected_loss(nqg_for("alice") + nqg_for("charlie") + nqg_for("dave"), nqg_for("eve"));
        assert_result_close(&result, user_list, &[("eve", expected_eve)]);
    }

    #[test]
    fn finds_previous_trust_list_from_non_consecutive_round() {
        let mut trusted_for_user_per_round: HashMap<u32, HashMap<String, Vec<String>>> = HashMap::new();

        let mut trust_44: HashMap<String, Vec<String>> = HashMap::new();
        trust_44.insert("alice".to_string(), trust_list(&["bob"]));
        trusted_for_user_per_round.insert(44, trust_44);

        let mut trust_40: HashMap<String, Vec<String>> = HashMap::new();
        trust_40.insert("alice".to_string(), trust_list(&["bob", "charlie"]));
        trusted_for_user_per_round.insert(40, trust_40);

        let user_list = &["alice", "bob", "charlie"];
        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round, nqg_map(user_list));
        let result = neuron.calculate_result(&users(user_list));

        let expected_charlie = expected_loss(nqg_for("alice"), nqg_for("charlie"));
        assert_result_close(&result, user_list, &[("charlie", expected_charlie)]);
    }

    #[test]
    fn uses_most_recent_previous_trust_list() {
        let mut trusted_for_user_per_round: HashMap<u32, HashMap<String, Vec<String>>> = HashMap::new();

        let mut trust_44: HashMap<String, Vec<String>> = HashMap::new();
        trust_44.insert("alice".to_string(), trust_list(&["bob"]));
        trusted_for_user_per_round.insert(44, trust_44);

        let mut trust_43: HashMap<String, Vec<String>> = HashMap::new();
        trust_43.insert("alice".to_string(), trust_list(&["bob", "charlie"]));
        trusted_for_user_per_round.insert(43, trust_43);

        let mut trust_42: HashMap<String, Vec<String>> = HashMap::new();
        trust_42.insert("alice".to_string(), trust_list(&["bob", "charlie", "dave"]));
        trusted_for_user_per_round.insert(42, trust_42);

        let user_list = &["alice", "bob", "charlie", "dave"];
        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round, nqg_map(user_list));
        let result = neuron.calculate_result(&users(user_list));

        // only charlie removed (comparing against round 43, not 42)
        let expected_charlie = expected_loss(nqg_for("alice"), nqg_for("charlie"));
        assert_result_close(&result, user_list, &[("charlie", expected_charlie)]);
    }

    #[test]
    fn user_had_empty_trust_list_before() {
        let mut trusted_for_user_per_round: HashMap<u32, HashMap<String, Vec<String>>> = HashMap::new();

        let mut trust_44: HashMap<String, Vec<String>> = HashMap::new();
        trust_44.insert("alice".to_string(), trust_list(&["bob", "charlie"]));
        trusted_for_user_per_round.insert(44, trust_44);

        let mut trust_43: HashMap<String, Vec<String>> = HashMap::new();
        trust_43.insert("alice".to_string(), trust_list(&[]));
        trusted_for_user_per_round.insert(43, trust_43);

        let user_list = &["alice", "bob", "charlie"];
        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round, nqg_map(user_list));
        let result = neuron.calculate_result(&users(user_list));
        assert_result_close(&result, user_list, &[]);
    }

    #[test]
    fn no_current_round_data_at_all() {
        let mut trusted_for_user_per_round: HashMap<u32, HashMap<String, Vec<String>>> = HashMap::new();

        let mut trust_43: HashMap<String, Vec<String>> = HashMap::new();
        trust_43.insert("alice".to_string(), trust_list(&["bob", "charlie"]));
        trusted_for_user_per_round.insert(43, trust_43);

        let user_list = &["alice", "bob", "charlie"];
        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round, nqg_map(user_list));
        let result = neuron.calculate_result(&users(user_list));
        assert_result_close(&result, user_list, &[]);
    }

    #[test]
    fn mixed_gains_and_losses() {
        let mut trusted_for_user_per_round: HashMap<u32, HashMap<String, Vec<String>>> = HashMap::new();

        let mut trust_44: HashMap<String, Vec<String>> = HashMap::new();
        trust_44.insert("alice".to_string(), trust_list(&["bob", "dave"]));
        trusted_for_user_per_round.insert(44, trust_44);

        let mut trust_43: HashMap<String, Vec<String>> = HashMap::new();
        trust_43.insert("alice".to_string(), trust_list(&["bob", "charlie"]));
        trusted_for_user_per_round.insert(43, trust_43);

        let user_list = &["alice", "bob", "charlie", "dave"];
        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round, nqg_map(user_list));
        let result = neuron.calculate_result(&users(user_list));

        let expected_charlie = expected_loss(nqg_for("alice"), nqg_for("charlie"));
        assert_result_close(&result, user_list, &[("charlie", expected_charlie)]);
    }

    #[test]
    fn user_in_users_but_not_in_trust_data() {
        let mut trusted_for_user_per_round: HashMap<u32, HashMap<String, Vec<String>>> = HashMap::new();

        let mut trust_44: HashMap<String, Vec<String>> = HashMap::new();
        trust_44.insert("alice".to_string(), trust_list(&["bob"]));
        trusted_for_user_per_round.insert(44, trust_44);

        let mut trust_43: HashMap<String, Vec<String>> = HashMap::new();
        trust_43.insert("alice".to_string(), trust_list(&["bob", "charlie"]));
        trusted_for_user_per_round.insert(43, trust_43);

        let user_list = &["alice", "bob", "charlie", "unknown_user"];
        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round, nqg_map(user_list));
        let result = neuron.calculate_result(&users(user_list));

        let expected_charlie = expected_loss(nqg_for("alice"), nqg_for("charlie"));
        assert_result_close(&result, user_list, &[("charlie", expected_charlie)]);
    }

    #[test]
    fn all_users_appear_in_output_even_with_zero_change() {
        let mut trusted_for_user_per_round: HashMap<u32, HashMap<String, Vec<String>>> = HashMap::new();

        let mut trust_44: HashMap<String, Vec<String>> = HashMap::new();
        trust_44.insert("alice".to_string(), trust_list(&["bob"]));
        trusted_for_user_per_round.insert(44, trust_44);

        let mut trust_43: HashMap<String, Vec<String>> = HashMap::new();
        trust_43.insert("alice".to_string(), trust_list(&["bob", "charlie"]));
        trusted_for_user_per_round.insert(43, trust_43);

        let user_list = &["alice", "bob", "charlie", "dave", "eve"];
        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round, nqg_map(user_list));
        let result = neuron.calculate_result(&users(user_list));

        assert_eq!(result.len(), 5);
        let expected_charlie = expected_loss(nqg_for("alice"), nqg_for("charlie"));
        assert_result_close(&result, user_list, &[("charlie", expected_charlie)]);
    }
}
