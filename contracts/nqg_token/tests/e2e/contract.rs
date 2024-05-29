use soroban_sdk::testutils::Address as AddressTrait;
use soroban_sdk::{Address, Env, Map, String, I256};

use crate::e2e::common::contract_utils::{deploy_contract, deploy_nqg_contract};

#[test]
fn updating_balances() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let governance_client = deploy_nqg_contract(&env, &admin);
    let nqg_token_client = deploy_contract(&env, &governance_client.address, &admin);

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
