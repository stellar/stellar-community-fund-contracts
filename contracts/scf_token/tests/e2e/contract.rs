use scf_token::{DataKey, SCFToken, SCFTokenClient, DECIMALS};
use soroban_sdk::testutils::Address as AddressTrait;
use soroban_sdk::xdr::{ScErrorCode, ScErrorType};
use soroban_sdk::{Address, Env, Error, Map, String, I256};

use crate::e2e::common::contract_utils::{
    deploy_and_setup, deploy_contract, deploy_scf_contract, update_balance, Deployment,
};

#[test]
fn initializing_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let governance_client = deploy_scf_contract(&env, &admin);
    let scf_token_address = env.register_contract(None, SCFToken);
    let scf_token_client = SCFTokenClient::new(&env, &scf_token_address);

    scf_token_client.initialize(&admin, &governance_client.address);
    // Try initializing again
    assert_eq!(
        scf_token_client.try_initialize(&admin, &governance_client.address),
        Err(Ok(Error::from_type_and_code(
            ScErrorType::Context,
            ScErrorCode::InvalidAction
        )))
    );
}

#[test]
fn updating_balances() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let governance_client = deploy_scf_contract(&env, &admin);
    let scf_token_client = deploy_contract(&env, &governance_client.address, &admin);

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
    scf_token_client.update_balance(&address);

    assert_eq!(scf_token_client.balance(&address), 10_i128.pow(DECIMALS));
}

#[test]
fn negative_nqg_score() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let Deployment {
        client,
        governance_client,
        address,
    } = deploy_and_setup(&env, &admin);

    env.mock_all_auths();
    update_balance(
        &env,
        &client,
        &governance_client,
        &address,
        -10 * 10_i128.pow(18),
    );

    assert_eq!(client.balance(&address), 0);
}

#[test]
fn updating_governance_address() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let Deployment {
        client,
        governance_client,
        ..
    } = deploy_and_setup(&env, &admin);

    env.mock_all_auths();

    let governance_address: Address = env.as_contract(&client.address, || {
        env.storage()
            .instance()
            .get(&DataKey::GovernanceAddress)
            .unwrap()
    });
    assert_eq!(governance_client.address, governance_address);

    let new_address = Address::generate(&env);
    client.set_governance_contract_address(&new_address);

    let governance_address: Address = env.as_contract(&client.address, || {
        env.storage()
            .instance()
            .get(&DataKey::GovernanceAddress)
            .unwrap()
    });
    assert_eq!(new_address, governance_address);
}
