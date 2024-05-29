use soroban_sdk::testutils::Address as AddressTrait;
use soroban_sdk::{Address, Env};

use crate::e2e::common::contract_utils::{deploy_and_setup, Deployment};

mod mock_contract {
    soroban_sdk::contractimport!(file = "../target/wasm32-unknown-unknown/release/mocks.wasm");
}

#[test]
fn upgrade_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let hash = env.deployer().upload_contract_wasm(mock_contract::WASM);

    let Deployment { client, .. } = deploy_and_setup(&env, &admin);
    let address = client.address.clone();

    client.upgrade(&hash);

    let new_contract_client = mock_contract::Client::new(&env, &address);
    assert!(new_contract_client.is_mock());
}
