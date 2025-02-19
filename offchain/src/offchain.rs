use std::fs;

use governance::{types::Vote, LayerAggregator, VotingSystem, VotingSystemClient};
use serde_json::{Map, Number, Value};
use soroban_sdk::testutils::Address as AddressTrait;
use soroban_sdk::{
    vec as SorobanVecMacro, Address, Env, Map as SorobanMap, String as SorobanString,
    Vec as SorobanVec, I256,
};
const ROUND: &'_ u32 = &31;

pub fn deploy_contract_without_initialization(env: &Env) -> VotingSystemClient {
    let contract_id = env.register(VotingSystem,());
    let contract_client = VotingSystemClient::new(env, &contract_id);

    contract_client
}

pub fn deploy_contract(env: &Env) -> VotingSystemClient {
    let contract_client = deploy_contract_without_initialization(env);

    env.mock_all_auths();
    let admin = Address::generate(env);
    contract_client.initialize(&admin, &ROUND);

    contract_client
}

pub fn setup_layers(env: &Env, contract_client: &VotingSystemClient<'_>) {
    let neurons_sum = SorobanVecMacro![
        &env,
        (
            SorobanString::from_str(&env, "TrustGraph"),
            I256::from_i128(&env, 1_000_000_000_000_000_000),
        ),
        (
            SorobanString::from_str(&env, "AssignedReputation"),
            I256::from_i128(&env, 1_000_000_000_000_000_000),
        ),
    ];
    contract_client.add_layer(&neurons_sum, &LayerAggregator::Sum);

    let neurons_product = SorobanVecMacro![
        &env,
        (
            SorobanString::from_str(&env, "PriorVotingHistory"),
            I256::from_i128(&env, 1_000_000_000_000_000_000),
        ),
    ];
    contract_client.add_layer(&neurons_product, &LayerAggregator::Product);
}

pub fn manual_tally(
    env: &Env,
    submissions: SorobanVec<(SorobanString, SorobanString)>,
    normalized_votes: SorobanMap<SorobanString, SorobanMap<SorobanString, Vote>>,
    trust_graph_neuron_result: SorobanMap<SorobanString, I256>,
    assigned_reputation_neuron_result: SorobanMap<SorobanString, I256>,
    prior_voting_history_neuron_result: SorobanMap<SorobanString, I256>,
) {
    // setup contract
    let contract_client: VotingSystemClient<'_> = deploy_contract(&env);
    setup_layers(&env, &contract_client);

    // set submissions
    contract_client.set_submissions(&submissions);

    // set normalized votes for each submission
    for (submission_id, votes) in normalized_votes.clone() {
        contract_client.set_votes_for_submission(&submission_id, &votes);
    }

    // set trust graph neuron results
    contract_client.set_neuron_result(
        &SorobanString::from_str(&env, "0"),
        &SorobanString::from_str(&env, "0"),
        &trust_graph_neuron_result,
    );

    // set assigned reputation neuron results
    contract_client.set_neuron_result(
        &SorobanString::from_str(&env, "0"),
        &SorobanString::from_str(&env, "1"),
        &assigned_reputation_neuron_result,
    );

    // set prior voting history neuron results
    contract_client.set_neuron_result(
        &SorobanString::from_str(&env, "1"),
        &SorobanString::from_str(&env, "0"),
        &prior_voting_history_neuron_result,
    );

    contract_client.calculate_voting_powers();

    // tally & write result to file
    let mut results_map: Map<String, Value> = Map::new();
    for (submission_id, _votes) in normalized_votes {
        let submission_id_string = submission_id.to_string();
        let result: i128 = match contract_client.tally_submission(&submission_id).to_i128() {
            Some(result) => result,
            None => panic!("i256 result of [{submission_id_string}] overflow i128"),
        };
        results_map.insert(submission_id_string, Value::String(result.to_string()));
    }
    let serialized = serde_json::to_string(&results_map).unwrap();
    fs::write(format!("result/voting_result.json"), serialized).unwrap();

    // save voting powers to file
    let mut powers_map: Map<String, Value> = Map::new();
    for (public_key, voting_power) in contract_client.get_voting_powers() {
        let public_key_string = public_key.to_string();
        let power = match voting_power.to_i128() {
            Some(power) => match Number::from_i128(power) {
                Some(number) => number,
                None => panic!("i256 to Number fail [{public_key_string}]"),
            },
            None => panic!("i256 voting power of [{public_key_string}] overflow i128"),
        };
        powers_map.insert(public_key_string, Value::Number(power));
    }
    let serialized = serde_json::to_string(&powers_map).unwrap();
    fs::write(format!("result/voting_powers.json"), serialized).unwrap();
}
