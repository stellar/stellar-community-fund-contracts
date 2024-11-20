use governance::{LayerAggregator, VotingSystem, VotingSystemClient, types::Vote};
use soroban_sdk::log;
use soroban_sdk::testutils::Address as AddressTrait;
use soroban_sdk::{map, vec, Address, Env, Map, String, Vec, I256};
pub fn deploy_contract_without_initialization(env: &Env) -> VotingSystemClient {
    let contract_id = env.register_contract(None, VotingSystem);
    let contract_client = VotingSystemClient::new(env, &contract_id);

    contract_client
}

pub fn deploy_contract(env: &Env) -> VotingSystemClient {
    let contract_client = deploy_contract_without_initialization(env);

    env.mock_all_auths();
    let admin = Address::generate(env);
    contract_client.initialize(&admin, &30);

    contract_client
}

fn offchain_manual_tally() {
    // setup contract
    let env = Env::default();
    env.budget().reset_unlimited();
    let contract_client = deploy_contract(&env);

    // setup neural governance
    let neurons_sum = vec![
        &env,
        (
            String::from_str(&env, "TrustGraph"),
            I256::from_i128(&env, 1_000_000_000_000_000_000),
        ),
        (
            String::from_str(&env, "AssignedReputation"),
            I256::from_i128(&env, 1_000_000_000_000_000_000),
        ),
    ];
    contract_client.add_layer(&neurons_sum, &LayerAggregator::Sum);

    let neurons_product = vec![
        &env,
        (
            String::from_str(&env, "PriorVotingHistory"),
            I256::from_i128(&env, 1_000_000_000_000_000_000),
        ),
    ];
    contract_client.add_layer(&neurons_product, &LayerAggregator::Product);

    // migrate submissions
    let new_submissions_raw: Vec<(String, String)> = vec![&env];
    contract_client.set_submissions(&new_submissions_raw);

    // upload normalized votes
    let normalized_submissions_votes: Map<String, Map<String, Vote>> = map![&env];
    for (submission_id, votes) in normalized_submissions_votes.clone() {
        contract_client.set_votes_for_submission(&submission_id, &votes);
    }

    // upload trust graph neuron results
    let trust_graph_values = vec![&env];
    let mut trust_graph_neuron_map: Map<String, I256> = Map::new(&env);
    for (key, value) in trust_graph_values {
        trust_graph_neuron_map.set(key, value);
    }

    contract_client.set_neuron_result(
        &String::from_str(&env, "0"),
        &String::from_str(&env, "0"),
        &trust_graph_neuron_map,
    );

    // upload trust graph neuron results
    let assigned_reputation_values = vec![&env];
    let mut assigned_reputation_neuron_map: Map<String, I256> = Map::new(&env);
    for (key, value) in assigned_reputation_values {
        assigned_reputation_neuron_map.set(key, value);
    }

    contract_client.set_neuron_result(
        &String::from_str(&env, "0"),
        &String::from_str(&env, "1"),
        &assigned_reputation_neuron_map,
    );

    // upload prior voting history neuron results
    let prior_voting_history_values = vec![&env];
    let mut prior_voting_history_neuron_map: Map<String, I256> = Map::new(&env);
    for (key, value) in prior_voting_history_values {
        prior_voting_history_neuron_map.set(key, value);
    }

    contract_client.set_neuron_result(
        &String::from_str(&env, "1"),
        &String::from_str(&env, "0"),
        &prior_voting_history_neuron_map,
    );

    contract_client.calculate_voting_powers();

    // tally
    for (submission_id, _votes) in normalized_submissions_votes {
        let result = contract_client.tally_submission(&submission_id);
        log!(&env, "result", submission_id, result);
    }
}
