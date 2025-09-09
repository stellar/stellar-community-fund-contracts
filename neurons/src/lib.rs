pub mod assigned_reputation;
pub mod neurons;
pub mod prior_voting_history;
pub mod quorum;
pub mod trust_graph;
pub mod trust_history;
pub mod types;
use assigned_reputation::{AssignedReputationNeuron, ReputationTier};
use neurons::Neuron;
use prior_voting_history::PriorVotingHistoryNeuron;
use quorum::{normalize_votes, DelegateesForUser};
use std::collections::HashMap;
use trust_graph::TrustGraphNeuron;
use trust_history::TrustHistoryNeuron;
use types::{Submission, Vote, DECIMALS};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run_neurons(
    current_round: u32,
    users_base: &str,
    previous_rounds_for_users: &str,
    users_reputation: &str,
    users_discord_roles: &str,
    trusted_for_user_per_round: &str,
) -> Result<String, String> {
    // parse all data
    // depending on which file is passed here, different users-base will be run through neurons
    let users_base: Vec<String> = match serde_json::from_str(users_base) {
        Ok(users_base) => users_base,
        Err(err) => return Err(format!("users_base json parsing error {}", err.to_string())),
    };
    let previous_rounds_for_users: HashMap<String, Vec<u32>> = match serde_json::from_str(previous_rounds_for_users) {
        Ok(previous_rounds_for_users) => previous_rounds_for_users,
        Err(err) => return Err(format!("previous_rounds_for_users json parsing error {}", err.to_string())),
    };
    let users_reputation: HashMap<String, ReputationTier> = match serde_json::from_str(users_reputation) {
        Ok(users_reputation) => users_reputation,
        Err(err) => return Err(format!("users_reputation json parsing error {}", err.to_string())),
    };
    let users_discord_roles: HashMap<String, Vec<String>> = match serde_json::from_str(users_discord_roles) {
        Ok(users_discord_roles) => users_discord_roles,
        Err(err) => return Err(format!("users_discord_roles json parsing error {}", err.to_string())),
    };
    println!("users_discord_roles: {:?}", users_discord_roles);
    let trusted_for_user_per_round: HashMap<u32, HashMap<String, Vec<String>>> =
        match serde_json::from_str(trusted_for_user_per_round) {
            Ok(trusted_for_user_per_round) => trusted_for_user_per_round,
            Err(err) => return Err(format!("trusted_for_user_per_round json parsing error {}", err.to_string())),
        };
    // create neurons
    let prior_voting_history_neuron = PriorVotingHistoryNeuron::from_data(previous_rounds_for_users);

    let assigned_reputation_neuron = AssignedReputationNeuron::from_data(users_reputation, users_discord_roles);

    // prepare and run trust neurons for previous rounds
    let mut trust_graph_neurons: Vec<Box<dyn Neuron>> = vec![];
    trusted_for_user_per_round.iter().for_each(|(round, trusted_for_user)| {
        if *round == current_round || *round == current_round - 1 {
            trust_graph_neurons.push(Box::new(TrustGraphNeuron::from_data(trusted_for_user.clone(), *round)));
        }
    });

    let trust_graph_neurons_results: HashMap<String, HashMap<String, f64>> =
        calculate_trust_neuron_results(&users_base, trust_graph_neurons);

    let trust_history_neuron = TrustHistoryNeuron::from_data(current_round as usize, trust_graph_neurons_results);

    // run all neurons
    let results = calculate_neuron_results(
        &users_base,
        vec![
            Box::new(prior_voting_history_neuron),
            Box::new(assigned_reputation_neuron),
            Box::new(trust_history_neuron),
        ],
    );

    Ok(serde_json::to_string_pretty(&results).unwrap())
}

#[wasm_bindgen]
pub fn run_votes_normalization(votes: &str, submissions: &str, delegatees_for_user: &str) -> Result<String, String> {
    let votes: HashMap<String, HashMap<String, Vote>> = match serde_json::from_str(votes) {
        Ok(votes) => votes,
        Err(err) => return Err(format!("votes json parsing error {}", err.to_string())),
    };

    let submissions: Vec<Submission> = match serde_json::from_str(submissions) {
        Ok(submissions) => submissions,
        Err(err) => return Err(format!("submissions json parsing error {}", err.to_string())),
    };

    let delegatees_for_user: HashMap<String, DelegateesForUser> = match serde_json::from_str(delegatees_for_user) {
        Ok(delegatees_for_user) => delegatees_for_user,
        Err(err) => return Err(format!("delegatees_for_user json parsing error {}", err.to_string())),
    };
    let normalized_votes = match normalize_votes(votes, &submissions, &delegatees_for_user) {
        Ok(normalized_votes) => normalized_votes,
        Err(err) => return Err(format!("error normalizing votes {}", err.to_string())),
    };
    Ok(serde_json::to_string_pretty(&normalized_votes).unwrap())
}

fn calculate_trust_neuron_results(
    users: &[String],
    neurons: Vec<Box<dyn Neuron>>,
) -> HashMap<String, HashMap<String, f64>> {
    let mut results: HashMap<String, HashMap<String, f64>> = HashMap::new();
    for neuron in neurons {
        // TODO maybe add fixed point decimal like in calculate_neuron_results()
        let result = neuron.calculate_result(users);
        results.insert(neuron.name(), result);
    }
    results
}

fn calculate_neuron_results(
    users: &[String],
    neurons: Vec<Box<dyn Neuron>>,
) -> HashMap<String, HashMap<String, String>> {
    let mut results: HashMap<String, HashMap<String, String>> = HashMap::new();
    for neuron in neurons {
        println!("running {}", neuron.name());
        let result = neuron.calculate_result(users);
        let result: HashMap<String, String> =
            result.into_iter().map(|(key, value)| (key, to_fixed_point_decimal(value).to_string())).collect();
        results.insert(neuron.name(), result);
    }
    results
}

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn to_fixed_point_decimal(val: f64) -> i128 {
    (val * DECIMALS as f64) as i128
}
