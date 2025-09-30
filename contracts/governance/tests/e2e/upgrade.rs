use crate::e2e::common::contract_utils::deploy_contract;
use soroban_sdk::{Env, Map, String, I256};

mod mock_contract {
    soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release/mocks.wasm");
}

#[test]
fn upgrade_contract() {
    let env = Env::default();
    let hash = env.deployer().upload_contract_wasm(mock_contract::WASM);

    let contract_client = deploy_contract(&env);
    let address = contract_client.address.clone();

    contract_client.upgrade(&hash);

    let new_contract_client = mock_contract::Client::new(&env, &address);
    assert!(new_contract_client.is_mock());
}

#[test]
fn storage_is_retained_after_upgrade() {
    let env = Env::default();
    let hash = env.deployer().upload_contract_wasm(mock_contract::WASM);

    let contract_client = deploy_contract(&env);
    let address = contract_client.address.clone();

    // Store data using old impl
    let mut result = Map::new(&env);
    result.set(String::from_str(&env, "user1"), I256::from_i32(&env, 100));
    result.set(String::from_str(&env, "user2"), I256::from_i32(&env, 200));

    contract_client.set_neuron_result(
        &String::from_str(&env, "0"),
        &String::from_str(&env, "0"),
        &result,
    );

    // Upgrade the contract
    contract_client.upgrade(&hash);
    let new_contract_client = mock_contract::Client::new(&env, &address);

    // Check if data is persisted
    let new_result = new_contract_client.get_stored_neuron_result();
    assert_eq!(result, new_result);
}
