use camino::Utf8Path;
use neurons::neurons::assigned_reputation::AssignedReputationNeuron;
use neurons::neurons::prior_voting_history::PriorVotingHistoryNeuron;
use neurons::neurons::trust_graph::TrustGraphNeuron;
use neurons::neurons::Neuron;
use neurons::quorum::{normalize_votes, DelegateesForUser};
use neurons::{Submission, Vote};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::fs;

pub const DECIMALS: i64 = 1_000_000_000_000_000_000;

fn write_result<T>(file_name: &str, data: &T)
where
    T: Serialize,
{
    let serialized = serde_json::to_string(&data).unwrap();
    fs::write(file_name, serialized).unwrap();
}

fn to_sorted_map<K, L>(data: HashMap<K, L>) -> BTreeMap<K, L>
where
    K: Ord,
{
    data.into_iter().collect()
}

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn to_fixed_point_decimal(val: f64) -> i128 {
    (val * DECIMALS as f64) as i128
}

fn calculate_neuron_results(users: &[String], neurons: Vec<Box<dyn Neuron>>) {
    for neuron in neurons {
        let result = neuron.calculate_result(users);
        let result: HashMap<String, String> = result
            .into_iter()
            .map(|(key, value)| (key, to_fixed_point_decimal(value).to_string()))
            .collect();
        let result = to_sorted_map(result);
        write_result(&format!("result/{}.json", neuron.name()), &result);
    }
}

fn main() {
    let path = Utf8Path::new("data/previous_rounds_for_users.json");
    let prior_voting_history_neuron = PriorVotingHistoryNeuron::try_from_file(path).unwrap();

    let path = Utf8Path::new("data/users_reputation.json");
    let assigned_reputation_neuron = AssignedReputationNeuron::try_from_file(path).unwrap();

    let path = Utf8Path::new("data/trusted_for_user.json");
    let trust_graph_neuron = TrustGraphNeuron::try_from_file(path).unwrap();

    let users_raw = fs::read_to_string("data/voters.json").unwrap();
    let users: Vec<String> = serde_json::from_str(users_raw.as_str()).unwrap();

    let votes_raw = fs::read_to_string("data/votes.json").unwrap();
    let votes: HashMap<String, HashMap<String, Vote>> =
        serde_json::from_str(votes_raw.as_str()).unwrap();

    let submissions_raw = fs::read_to_string("data/submissions.json").unwrap();
    let submissions: Vec<Submission> = serde_json::from_str(submissions_raw.as_str()).unwrap();

    let delegatees_for_user_raw = fs::read_to_string("data/delegatees_for_user.json").unwrap();
    let delegatees_for_user: HashMap<String, DelegateesForUser> =
        serde_json::from_str(delegatees_for_user_raw.as_str()).unwrap();
    let normalized_votes = normalize_votes(votes, &submissions, &delegatees_for_user).unwrap();
    write_result(
        "result/normalized_votes.json",
        &to_sorted_map(normalized_votes),
    );

    calculate_neuron_results(
        &users,
        vec![
            Box::new(prior_voting_history_neuron),
            Box::new(assigned_reputation_neuron),
            Box::new(trust_graph_neuron),
        ],
    );
}
