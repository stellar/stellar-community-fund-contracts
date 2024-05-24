use soroban_sdk::{Address, Env, I256, Map, String, vec};
use soroban_sdk::testutils::Address as AddressTrait;
use governor_wrapper::GovernorWrapper;
use governor_wrapper::GovernorWrapperClient;
use crate::governance::LayerAggregator;

mod governance {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/governance.wasm");
}

mod admin_votes {
    use soroban_sdk::contractimport;

    contractimport!(file = "./soroban_admin_votes.wasm");
}

#[test]
fn updating_balances() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let admin_votes_address = env.register_contract_wasm(None, admin_votes::WASM);
    let admin_votes_client = admin_votes::Client::new(&env, &admin_votes_address);

    let governance_address = env.register_contract_wasm(None, governance::WASM);
    let governance_client = governance::Client::new(&env, &governance_address);

    let governor_wrapper_address = env.register_contract(None, GovernorWrapper);
    let governor_wrapper_client = GovernorWrapperClient::new(&env, &governor_wrapper_address);

    admin_votes_client.initialize(&admin, &admin, &0, &String::from_str(&env, "test token"), &String::from_str(&env, "test"));

    governance_client.initialize(&admin, &25);

    governor_wrapper_client.initialize(&admin, &admin_votes_address, &governance_address);

    let neurons = vec![&env, (String::from_str(&env, "Layer1"), I256::from_i128(&env, 10_i128.pow(18)))];
    governance_client.add_layer(&neurons, &LayerAggregator::Sum);

    let address = Address::generate(&env);
    let mut result = Map::new(&env);
    result.set(address.to_string(), I256::from_i128(&env, 10_i128.pow(18)));

    governance_client.set_neuron_result(&String::from_str(&env, "0"), &String::from_str(&env, "0"), &result);

    governance_client.calculate_voting_powers();

    env.budget().reset_default();
    governor_wrapper_client.update_balance(&address);

    assert_eq!(admin_votes_client.get_votes(&address), 1);
    assert_eq!(admin_votes_client.balance(&address), 1);
}