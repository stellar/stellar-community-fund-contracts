pub mod assigned_reputation;
pub mod neurons;
pub mod prior_voting_history;
pub mod quorum;
pub mod retro_vote_quality;
pub mod trust_graph;
pub mod trust_loss;
pub mod types;
use assigned_reputation::{AssignedReputationNeuron, ReputationTier};
use neurons::Neuron;
use prior_voting_history::PriorVotingHistoryNeuron;
use quorum::{normalize_votes, DelegateesForUser};
use std::collections::{HashMap, HashSet};
use trust_graph::TrustGraphNeuron;
use types::{Vote, DECIMALS};
use wasm_bindgen::prelude::*;

use crate::{retro_vote_quality::RetroVoteQualityNeuron, trust_loss::TrustLossNeuron};

#[wasm_bindgen]
pub fn run_neurons(
    current_round: u32,
    users_base: &str,
    previous_rounds_for_users: &str,
    users_reputation: &str,
    users_discord_roles: &str,
    trusted_for_user_per_round: &str,
    votes_per_round: &str,
    normalized_votes_per_round: &str,
    tranche_status_map: &str,
    submissions_airtable_ids: &str,
    submitters_per_round: &str,
) -> Result<String, String> {
    // parse all data
    // depending on which file is passed here, different users-base will be run through neurons
    let users_base: Vec<String> = match serde_json::from_str(users_base) {
        Ok(users_base) => users_base,
        Err(err) => return Err(format!("users_base json parsing error {}", err.to_string())),
    };
    let normalized_votes_per_round: HashMap<u32, HashMap<String, HashMap<String, Vote>>> = match serde_json::from_str(normalized_votes_per_round) {
        Ok(normalized_votes_per_round) => normalized_votes_per_round,
        Err(err) => return Err(format!("normalized_votes_per_round json parsing error {}", err.to_string())),
    };
    let tranche_status_map: HashMap<String, Vec<String>> = match serde_json::from_str(tranche_status_map) {
        Ok(tranche_status_map) => tranche_status_map,
        Err(err) => return Err(format!("tranche_status_map json parsing error {}", err.to_string())),
    };
    let submissions_airtable_ids: HashMap<String, String> = match serde_json::from_str(submissions_airtable_ids) {
        Ok(submissions_airtable_ids) => submissions_airtable_ids,
        Err(err) => return Err(format!("submissions_airtable_ids json parsing error {}", err.to_string())),
    };

    let previous_rounds_for_users: HashMap<String, Vec<u32>> = match serde_json::from_str(previous_rounds_for_users) {
        Ok(previous_rounds_for_users) => previous_rounds_for_users,
        Err(err) => return Err(format!("previous_rounds_for_users json parsing error {}", err.to_string())),
    };
    let votes_per_round: HashMap<u32, HashMap<String, HashMap<String, Vote>>> = match serde_json::from_str(votes_per_round) {
        Ok(votes_per_round) => votes_per_round,
        Err(err) => return Err(format!("votes_per_round json parsing error {}", err.to_string())),
    };
    let users_reputation: HashMap<String, ReputationTier> = match serde_json::from_str(users_reputation) {
        Ok(users_reputation) => users_reputation,
        Err(err) => return Err(format!("users_reputation json parsing error {}", err.to_string())),
    };
    let users_discord_roles: HashMap<String, Vec<String>> = match serde_json::from_str(users_discord_roles) {
        Ok(users_discord_roles) => users_discord_roles,
        Err(err) => return Err(format!("users_discord_roles json parsing error {}", err.to_string())),
    };
    let trusted_for_user_per_round: HashMap<u32, HashMap<String, Vec<String>>> = match serde_json::from_str(trusted_for_user_per_round) {
        Ok(trusted_for_user_per_round) => trusted_for_user_per_round,
        Err(err) => return Err(format!("trusted_for_user_per_round json parsing error {}", err.to_string())),
    };
    let submitters_per_round: HashMap<u32, HashSet<String>> = match serde_json::from_str(submitters_per_round) {
        Ok(submitters_per_round) => submitters_per_round,
        Err(err) => return Err(format!("submitters_per_round json parsing error {}", err.to_string())),
    };

    // Voting History
    let prior_voting_history_neuron = PriorVotingHistoryNeuron::from_data(previous_rounds_for_users, votes_per_round.clone(), submitters_per_round, current_round);
    // Assigned Reputation
    let assigned_reputation_neuron = AssignedReputationNeuron::from_data(users_reputation, users_discord_roles);

    // Trust Graph
    let trusted_for_user_current_round = trusted_for_user_per_round.get(&current_round).unwrap();
    let trust_graph_neuron = TrustGraphNeuron::from_data(trusted_for_user_current_round.clone());

    // Trust Loss
    let trust_loss_neuron = TrustLossNeuron::from_data(current_round, trusted_for_user_per_round);

    // Retro Vote Quality
    let retro_vote_quality_neuron = RetroVoteQualityNeuron::from_data(votes_per_round, normalized_votes_per_round, tranche_status_map, submissions_airtable_ids);

    // Run all neurons
    let results = calculate_neuron_results(
        &users_base,
        vec![
            Box::new(prior_voting_history_neuron),
            Box::new(assigned_reputation_neuron),
            Box::new(retro_vote_quality_neuron),
            Box::new(trust_graph_neuron),
            Box::new(trust_loss_neuron),
        ],
    );

    Ok(serde_json::to_string_pretty(&results).unwrap())
}

#[wasm_bindgen]
pub fn run_votes_normalization(votes: &str, delegatees_for_user: &str) -> Result<String, String> {
    let votes: HashMap<String, HashMap<String, Vote>> = match serde_json::from_str(votes) {
        Ok(votes) => votes,
        Err(err) => return Err(format!("votes json parsing error {}", err.to_string())),
    };

    let delegatees_for_user: HashMap<String, DelegateesForUser> = match serde_json::from_str(delegatees_for_user) {
        Ok(delegatees_for_user) => delegatees_for_user,
        Err(err) => return Err(format!("delegatees_for_user json parsing error {}", err.to_string())),
    };

    let normalized_votes = match normalize_votes(votes, &delegatees_for_user) {
        Ok(normalized_votes) => normalized_votes,
        Err(err) => return Err(format!("error normalizing votes {}", err.to_string())),
    };
    Ok(serde_json::to_string_pretty(&normalized_votes).unwrap())
}

fn calculate_neuron_results(users: &[String], neurons: Vec<Box<dyn Neuron>>) -> HashMap<String, HashMap<String, String>> {
    let mut results: HashMap<String, HashMap<String, String>> = HashMap::new();
    for neuron in neurons {
        println!("running {}", neuron.name());
        let result = neuron.calculate_result(users);
        let result: HashMap<String, String> = result.into_iter().map(|(key, value)| (key, to_fixed_point_decimal(value).to_string())).collect();
        results.insert(neuron.name(), result);
    }
    results
}

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn to_fixed_point_decimal(val: f64) -> i128 {
    (val * DECIMALS as f64) as i128
}

#[cfg(test)]
mod tests {
    use super::*;

    // End-to-end orchestration: feed JSON strings through `run_neurons` and assert the five
    // named neurons each appear in the output with fixed-point-decimal string values.
    //
    // The trust graph is kept minimal and self-consistent (alice trusts bob; bob and carol trust
    // alice) so the highly-trusted bonus never references an absent user and the score sort never
    // sees NaN.
    #[test]
    fn run_neurons_outputs_all_five_named_neurons() {
        let result = run_neurons(
            30,
            r#"["alice","bob","carol"]"#,
            r#"{"alice":[29],"bob":[30],"carol":[]}"#,
            r#"{"alice":2,"bob":0,"carol":0}"#,
            r#"{"alice":["SDF"],"bob":[],"carol":[]}"#,
            r#"{"29":{"bob":["alice"]},"30":{"alice":["bob"],"bob":["alice"],"carol":["alice"]}}"#,
            "{}",
            "{}",
            "{}",
            "{}",
            "{}",
        )
        .expect("run_neurons should succeed");

        let parsed: HashMap<String, HashMap<String, String>> = serde_json::from_str(&result).expect("output should be valid JSON");

        for name in [
            "prior_voting_history_neuron",
            "assigned_reputation_neuron",
            "trust_graph_neuron",
            "trust_loss_neuron",
            "retro_vote_quality_neuron",
        ] {
            let entry = parsed.get(name).unwrap_or_else(|| panic!("missing neuron {name}"));
            for user in ["alice", "bob", "carol"] {
                assert!(entry.contains_key(user), "{name} missing {user}");
            }
        }

        // assigned_reputation is deterministic: alice = Navigator(2.0) + SDF role(1.0) = 3.0.
        let rep = &parsed["assigned_reputation_neuron"];
        assert_eq!(rep["alice"], to_fixed_point_decimal(3.0).to_string());
        assert_eq!(rep["bob"], to_fixed_point_decimal(0.0).to_string());

        let trust_graph = parsed.get("trust_graph_neuron").expect("missing trust_graph_neuron");

        assert_eq!(trust_graph["alice"], ((to_fixed_point_decimal(3.0) * 110) / 100).to_string());
        assert_eq!(trust_graph["carol"], to_fixed_point_decimal(0.0).to_string());
    }
}
