use crate::neurons::Neuron;
use std::collections::HashMap;

const HOW_DEEP: u32 = 32;

#[derive(Clone, Debug)]
pub struct TrustLossNeuron {
    round: u32,
    trusted_for_user_per_round: HashMap<u32, HashMap<String, Vec<String>>>,
}

impl TrustLossNeuron {
    pub fn from_data(round: u32, trusted_for_user_per_round: HashMap<u32, HashMap<String, Vec<String>>>) -> Self {
        Self { round, trusted_for_user_per_round }
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
}

impl Neuron for TrustLossNeuron {
    fn name(&self) -> String {
        format!("trust_loss_neuron")
    }

    fn calculate_result(&self, users: &[String]) -> HashMap<String, f64> {
        let mut trust_diff_map: HashMap<String, f64> = users.iter().map(|user| (user.to_string(), 0.0)).collect();
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
                    // only users present in the input slice are tracked in the output map
                    if let Some(value) = trust_diff_map.get_mut(trusted_user) {
                        *value -= 1.0;
                    }
                }
            });
        }
        trust_diff_map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn users(names: &[&str]) -> Vec<String> {
        names.iter().map(|x| x.to_string()).collect()
    }

    fn trust_list(names: &[&str]) -> Vec<String> {
        names.iter().map(|x| x.to_string()).collect()
    }

    fn expected_result(user_names: &[&str], overrides: &[(&str, f64)]) -> HashMap<String, f64> {
        let mut result: HashMap<String, f64> = user_names.iter().map(|name| (name.to_string(), 0.0)).collect();
        for (name, value) in overrides {
            result.insert(name.to_string(), *value);
        }
        result
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

        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round);
        let user_list = &["alice", "tom", "bob", "andy", "john", "adam"];

        assert_eq!(neuron.calculate_result(&users(user_list)), expected_result(user_list, &[]));
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

        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round);
        let user_list = &["alice", "tom", "bob", "andy", "john", "adam"];

        assert_eq!(neuron.calculate_result(&users(user_list)), expected_result(user_list, &[]));
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

        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round);
        let user_list = &["alice", "tom", "bob", "andy", "john"];

        assert_eq!(
            neuron.calculate_result(&users(user_list)),
            expected_result(user_list, &[("andy", -2.0), ("bob", -1.0)])
        );
    }

    #[test]
    fn name_returns_correct_value() {
        let neuron = TrustLossNeuron::from_data(44, HashMap::new());
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

        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round);
        assert_eq!(neuron.calculate_result(&[]), HashMap::new());
    }

    #[test]
    fn empty_trust_data_returns_all_users_with_zero() {
        let neuron = TrustLossNeuron::from_data(44, HashMap::new());
        let user_list = &["alice", "bob"];
        assert_eq!(neuron.calculate_result(&users(user_list)), expected_result(user_list, &[]));
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

        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round);
        let user_list = &["alice", "bob", "charlie"];
        let result = neuron.calculate_result(&users(user_list));
        assert_eq!(result, expected_result(user_list, &[]));
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

        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round);
        let user_list = &["alice", "bob", "charlie", "newuser"];
        let result = neuron.calculate_result(&users(user_list));

        assert_eq!(result, expected_result(user_list, &[("charlie", -1.0)]));
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

        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round);
        let user_list = &["alice", "bob"];
        let result = neuron.calculate_result(&users(user_list));
        assert_eq!(result, expected_result(user_list, &[]));
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

        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round);
        let user_list = &["alice", "bob", "charlie", "dave", "eve"];
        let result = neuron.calculate_result(&users(user_list));

        assert_eq!(result, expected_result(user_list, &[("eve", -3.0)]));
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

        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round);
        let user_list = &["alice", "bob", "charlie"];
        let result = neuron.calculate_result(&users(user_list));

        assert_eq!(result, expected_result(user_list, &[("charlie", -1.0)]));
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

        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round);
        let user_list = &["alice", "bob", "charlie", "dave"];
        let result = neuron.calculate_result(&users(user_list));

        assert_eq!(result, expected_result(user_list, &[("charlie", -1.0)]));
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

        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round);
        let user_list = &["alice", "bob", "charlie"];
        let result = neuron.calculate_result(&users(user_list));
        assert_eq!(result, expected_result(user_list, &[]));
    }

    #[test]
    fn no_current_round_data_at_all() {
        let mut trusted_for_user_per_round: HashMap<u32, HashMap<String, Vec<String>>> = HashMap::new();

        let mut trust_43: HashMap<String, Vec<String>> = HashMap::new();
        trust_43.insert("alice".to_string(), trust_list(&["bob", "charlie"]));
        trusted_for_user_per_round.insert(43, trust_43);

        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round);
        let user_list = &["alice", "bob", "charlie"];
        let result = neuron.calculate_result(&users(user_list));
        assert_eq!(result, expected_result(user_list, &[]));
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

        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round);
        let user_list = &["alice", "bob", "charlie", "dave"];
        let result = neuron.calculate_result(&users(user_list));

        assert_eq!(result, expected_result(user_list, &[("charlie", -1.0)]));
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

        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round);
        let user_list = &["alice", "bob", "charlie", "unknown_user"];
        let result = neuron.calculate_result(&users(user_list));

        assert_eq!(result, expected_result(user_list, &[("charlie", -1.0)]));
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

        let neuron = TrustLossNeuron::from_data(44, trusted_for_user_per_round);
        let user_list = &["alice", "bob", "charlie", "dave", "eve"];
        let result = neuron.calculate_result(&users(user_list));

        assert_eq!(result.len(), 5);
        for name in user_list {
            assert!(result.contains_key(*name));
        }
        assert_eq!(result["charlie"], -1.0);
        assert_eq!(result["alice"], 0.0);
        assert_eq!(result["bob"], 0.0);
        assert_eq!(result["dave"], 0.0);
        assert_eq!(result["eve"], 0.0);
    }
}
