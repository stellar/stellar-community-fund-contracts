use soroban_sdk::testutils::Address as AddressTrait;
use soroban_sdk::{Address, Env};

use crate::e2e::common::contract_utils::{
    deploy_and_setup, set_nqg_results, update_balance, Deployment,
};

#[test]
fn balance_round() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let Deployment {
        client,
        governance_client,
    } = deploy_and_setup(&env, &admin);
    env.mock_all_auths();

    let address = Address::generate(&env);

    governance_client.set_current_round(&30);
    set_nqg_results(&env, &governance_client, &address, 1 * 10_i128.pow(18));
    client.update_balance(&address);
    assert_eq!(client.balance_round(&address), 30);

    governance_client.set_current_round(&31);
    set_nqg_results(&env, &governance_client, &address, 2 * 10_i128.pow(18));
    client.update_balance(&address);
    assert_eq!(client.balance_round(&address), 31);

    governance_client.set_current_round(&33);
    set_nqg_results(&env, &governance_client, &address, 3 * 10_i128.pow(18));
    client.update_balance(&address);
    assert_eq!(client.balance_round(&address), 33);
}

#[test]
fn all_addresses() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let Deployment {
        client,
        governance_client,
    } = deploy_and_setup(&env, &admin);
    env.mock_all_auths();
    governance_client.set_current_round(&33);
    let mut addresses: Vec<Address> = vec![];
    for _ in 1..=10 {
        let addr = Address::generate(&env);
        set_nqg_results(&env, &governance_client, &addr, 10_i128.pow(18));
        client.update_balance(&addr);
        addresses.push(addr);
    }
    governance_client.set_current_round(&34);
    for addr in &addresses {
        set_nqg_results(&env, &governance_client, &addr, 10_i128.pow(18));
        client.update_balance(&addr);
    }
    // check for duplicates
    let fetched_addresses = client.all_addresses();
    let mut dedup: Vec<Address> = vec![];
    for addr in &fetched_addresses {
        assert!(!dedup.contains(&addr));
        dedup.push(addr);
    }
    // check if all required addresses where returned
    for a in &addresses {
        assert!(fetched_addresses.contains(a.clone()))
    }
    assert_eq!(addresses.len(), fetched_addresses.len() as usize);
}

#[test]
fn proposal_threshold_top_10_percent() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let Deployment {
        client,
        governance_client,
    } = deploy_and_setup(&env, &admin);
    env.mock_all_auths();

    let mut random_balances: Vec<i128> = (1..=100).collect();
    random_balances.reverse();
    for b in &random_balances {
        update_balance(
            &env,
            &client,
            &governance_client,
            &Address::generate(&env),
            b * 10_i128.pow(18),
        );
    }

    assert_eq!(client.optimal_threshold(), 91 * 10_i128.pow(9));
}

#[test]
fn proposal_threshold_fallback_5_users() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let Deployment {
        client,
        governance_client,
    } = deploy_and_setup(&env, &admin);
    env.mock_all_auths();

    let random_balances: Vec<i128> = vec![2, 4, 6, 8, 10, 1, 3, 5, 7, 9];
    for b in &random_balances {
        update_balance(
            &env,
            &client,
            &governance_client,
            &Address::generate(&env),
            b * 10_i128.pow(18),
        );
    }

    assert_eq!(client.optimal_threshold(), 6 * 10_i128.pow(9));
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn proposal_threshold_zero_users() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let Deployment {
        client,
        governance_client: _,
    } = deploy_and_setup(&env, &admin);
    env.mock_all_auths();

    let _ = client.optimal_threshold();
}

#[test]
fn proposal_threshold_less_than_5() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let Deployment {
        client,
        governance_client,
    } = deploy_and_setup(&env, &admin);
    env.mock_all_auths();

    let random_balances: Vec<i128> = vec![1, 2, 4, 6];
    for b in &random_balances {
        update_balance(
            &env,
            &client,
            &governance_client,
            &Address::generate(&env),
            b * 10_i128.pow(18),
        );
    }
    assert_eq!(client.optimal_threshold(), 1 * 10_i128.pow(9));
}
