use crate::governance::LayerAggregator;
use nqg_token::{NQGToken, NQGTokenClient};
use soroban_sdk::testutils::Address as AddressTrait;
use soroban_sdk::{vec, Address, Env, Map, String, I256};

mod governance {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/governance.wasm");
}

#[test]
fn updating_balances() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let governance_address = env.register_contract_wasm(None, governance::WASM);
    let governance_client = governance::Client::new(&env, &governance_address);

    let governor_wrapper_address = env.register_contract(None, NQGToken);
    let nqg_token_client = NQGTokenClient::new(&env, &governor_wrapper_address);

    governance_client.initialize(&admin, &25);

    let neurons = vec![
        &env,
        (
            String::from_str(&env, "Layer1"),
            I256::from_i128(&env, 10_i128.pow(18)),
        ),
    ];
    governance_client.add_layer(&neurons, &LayerAggregator::Sum);

    let address = Address::generate(&env);
    let mut result = Map::new(&env);
    result.set(address.to_string(), I256::from_i128(&env, 10_i128.pow(18)));

    governance_client.set_neuron_result(
        &String::from_str(&env, "0"),
        &String::from_str(&env, "0"),
        &result,
    );

    governance_client.calculate_voting_powers();

    env.budget().reset_default();
    nqg_token_client.update_balance(&address);

    assert_eq!(nqg_token_client.balance(&address), 1);
}
