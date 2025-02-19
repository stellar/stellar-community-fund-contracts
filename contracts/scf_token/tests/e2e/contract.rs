use scf_token::{DataKey, SCFToken, SCFTokenClient, DECIMALS};
use soroban_sdk::testutils::Address as AddressTrait;
use soroban_sdk::{Address, Env, Error, Map, String, I256};

use crate::e2e::common::contract_utils::{
    bump_round, deploy_and_setup, deploy_contract, deploy_scf_contract, set_nqg_results,
    update_balance, Deployment,
};

#[test]
fn initializing_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let governance_client = deploy_scf_contract(&env, &admin);
    let scf_token_address = env.register( SCFToken, ());
    let scf_token_client = SCFTokenClient::new(&env, &scf_token_address);

    scf_token_client.initialize(&admin, &governance_client.address);
    // Try initializing again
    assert_eq!(
        scf_token_client.try_initialize(&admin, &governance_client.address),
        Err(Ok(Error::from_contract_error(2)))
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

    env.cost_estimate().budget().reset_default();
    scf_token_client.update_balance(&address);

    assert_eq!(scf_token_client.balance(&address), 10_i128.pow(DECIMALS));
}

#[test]
fn updating_balance_is_allowed_only_once_per_round() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let Deployment {
        client,
        governance_client,
    } = deploy_and_setup(&env, &admin);
    env.mock_all_auths();

    let address = Address::generate(&env);

    set_nqg_results(&env, &governance_client, &address, 10_i128.pow(18));

    client.update_balance(&address);
    assert!(client.try_update_balance(&address).is_err());

    bump_round(&governance_client);
    set_nqg_results(&env, &governance_client, &address, 10_i128.pow(18));

    client.update_balance(&address);
}

#[test]
fn negative_nqg_score() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let Deployment {
        client,
        governance_client,
    } = deploy_and_setup(&env, &admin);

    let address = Address::generate(&env);

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
