use crate::e2e::common::contract_utils::{deploy_and_setup, jump, update_balance, Deployment};
use soroban_sdk::testutils::Address as AddressTrait;
use soroban_sdk::{Address, Env, Map, String, I256};

#[test]
fn checkpoints() {
    let mut env = Env::default();

    let admin = Address::generate(&env);
    let Deployment {
        client,
        governance_client,
        address,
    } = deploy_and_setup(&env, &admin);
    env.mock_all_auths();

    // Store current balances
    let sequence = env.ledger().sequence();
    let balance = client.balance(&address);

    // Update the balance in future ledgers
    jump(&mut env, 100);

    let mut result: Map<String, I256> = Map::new(&env);
    result.set(address.to_string(), I256::from_i128(&env, 20_i128.pow(20)));

    governance_client.set_neuron_result(
        &String::from_str(&env, "0"),
        &String::from_str(&env, "0"),
        &result,
    );
    governance_client.calculate_voting_powers();

    client.update_balance(&address);

    jump(&mut env, 1);

    let new_balance = client.balance(&address);

    // Verify history is preserved for votes
    assert_eq!(client.get_votes(&address), new_balance);
    assert_eq!(
        client.get_past_votes(&address, &(env.ledger().sequence() - 1)),
        new_balance
    );
    assert_eq!(
        client.get_past_votes(&address, &(env.ledger().sequence() - 100)),
        balance
    );
    assert_eq!(balance, 1000000000);
    assert_eq!(new_balance, 104857600000000000);

    // Verify history is preserved for total supply
    assert_eq!(client.total_supply(), new_balance);
    assert_eq!(
        client.get_past_total_supply(&(env.ledger().sequence() - 1)),
        new_balance
    );
    assert_eq!(
        client.get_past_total_supply(&(env.ledger().sequence() - 100)),
        balance
    );
}

#[test]
fn total_supply() {
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
        20 * 10_i128.pow(18),
    );
    assert_eq!(client.total_supply(), 20 * 10_i128.pow(9));
    update_balance(
        &env,
        &client,
        &governance_client,
        &address,
        30 * 10_i128.pow(18),
    );
    assert_eq!(client.total_supply(), 30 * 10_i128.pow(9));
    update_balance(
        &env,
        &client,
        &governance_client,
        &address,
        10 * 10_i128.pow(18),
    );
    assert_eq!(client.total_supply(), 10 * 10_i128.pow(9));
}

#[test]
fn total_supply_multiple_users() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let Deployment {
        client,
        governance_client,
        address,
    } = deploy_and_setup(&env, &admin);
    let address2 = Address::generate(&env);
    env.mock_all_auths();

    update_balance(
        &env,
        &client,
        &governance_client,
        &address,
        20 * 10_i128.pow(18),
    );
    update_balance(
        &env,
        &client,
        &governance_client,
        &address2,
        30 * 10_i128.pow(18),
    );
    assert_eq!(client.total_supply(), 50 * 10_i128.pow(9));

    update_balance(
        &env,
        &client,
        &governance_client,
        &address,
        10 * 10_i128.pow(18),
    );
    update_balance(
        &env,
        &client,
        &governance_client,
        &address2,
        20 * 10_i128.pow(18),
    );
    assert_eq!(client.total_supply(), 30 * 10_i128.pow(9));

    update_balance(
        &env,
        &client,
        &governance_client,
        &address,
        100 * 10_i128.pow(18),
    );
    update_balance(
        &env,
        &client,
        &governance_client,
        &address2,
        20 * 10_i128.pow(18),
    );
    assert_eq!(client.total_supply(), 120 * 10_i128.pow(9));
}
