use soroban_sdk::testutils::Address as AddressTrait;
use soroban_sdk::{Address, Env};

use crate::e2e::common::contract_utils::{deploy_and_setup, jump, update_balance, Deployment};

#[test]
fn checkpoints() {
    let mut env = Env::default();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);

    let Deployment {
        client,
        governance_client,
        ..
    } = deploy_and_setup(&env, &admin);
    env.mock_all_auths();

    let address = Address::generate(&env);
    update_balance(&env, &client, &governance_client, &address, 10_i128.pow(18));

    // Store current balances
    let balance = client.balance(&address);

    // Update the balance in future ledgers
    jump(&mut env, 100);

    governance_client.set_current_round(&(governance_client.get_current_round() + 1));
    update_balance(
        &env,
        &client,
        &governance_client,
        &address,
        2 * 10_i128.pow(18),
    );

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
    assert_eq!(balance, 1_000_000_000);
    assert_eq!(new_balance, 2_000_000_000);

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
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let Deployment {
        client,
        governance_client,
    } = deploy_and_setup(&env, &admin);
    env.mock_all_auths();

    let address = Address::generate(&env);

    update_balance(
        &env,
        &client,
        &governance_client,
        &address,
        20 * 10_i128.pow(18),
    );
    assert_eq!(client.total_supply(), 20 * 10_i128.pow(9));
    governance_client.set_current_round(&(governance_client.get_current_round() + 1));
    update_balance(
        &env,
        &client,
        &governance_client,
        &address,
        30 * 10_i128.pow(18),
    );
    assert_eq!(client.total_supply(), 30 * 10_i128.pow(9));
    governance_client.set_current_round(&(governance_client.get_current_round() + 1));
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
#[allow(clippy::too_many_lines)]
fn total_supply_multiple_users() {
    let env = Env::default();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let Deployment {
        client,
        governance_client,
    } = deploy_and_setup(&env, &admin);
    let address = Address::generate(&env);
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

    governance_client.set_current_round(&(governance_client.get_current_round() + 1));
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

    governance_client.set_current_round(&(governance_client.get_current_round() + 1));
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

    governance_client.set_current_round(&(governance_client.get_current_round() + 1));
    update_balance(
        &env,
        &client,
        &governance_client,
        &address,
        -100 * 10_i128.pow(18),
    );
    update_balance(
        &env,
        &client,
        &governance_client,
        &address2,
        20 * 10_i128.pow(18),
    );
    assert_eq!(client.total_supply(), 20 * 10_i128.pow(9));

    governance_client.set_current_round(&(governance_client.get_current_round() + 1));
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
        -20 * 10_i128.pow(18),
    );
    assert_eq!(client.total_supply(), 100 * 10_i128.pow(9));

    governance_client.set_current_round(&(governance_client.get_current_round() + 1));
    update_balance(
        &env,
        &client,
        &governance_client,
        &address,
        -100 * 10_i128.pow(18),
    );
    update_balance(
        &env,
        &client,
        &governance_client,
        &address2,
        -20 * 10_i128.pow(18),
    );
    assert_eq!(client.total_supply(), 0);
}
