use soroban_sdk::testutils::Address as AddressTrait;
use soroban_sdk::{Address, Env};

use crate::e2e::common::contract_utils::{deploy_and_setup, update_balance, Deployment};

#[test]
fn nth_top_balance() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let Deployment {
        client,
        governance_client,
    } = deploy_and_setup(&env, &admin);
    env.mock_all_auths();

    let random_balances: Vec<i128> = vec![7, 1, 15, 20, 6, 18, 10, 13, 16, 14];
    for b in &random_balances {
        update_balance(
            &env,
            &client,
            &governance_client,
            &Address::generate(&env),
            b * 10_i128.pow(18),
        );
    }

    assert_eq!(client.nth_top_balance(&3), 16 * 10_i128.pow(9));
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

    let random_balances: Vec<i128> = vec![2,4,6,8,10,1,3,5,7,9];
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

// TODO fix this test, it should assert that contract call fails with OutOfBounds error
// #[test]
// #[should_panic]
// fn proposal_threshold_out_of_bounds() {
//     let env = Env::default();
//     env.budget().reset_unlimited();

//     let admin = Address::generate(&env);
//     let Deployment {
//         client,
//         governance_client,
//     } = deploy_and_setup(&env, &admin);
//     env.mock_all_auths();

//     let random_balances: Vec<i128> = vec![2,4,6];
//     for b in &random_balances {
//         update_balance(
//             &env,
//             &client,
//             &governance_client,
//             &Address::generate(&env),
//             b * 10_i128.pow(18),
//         );
//     }
//     let _ = client.optimal_threshold();
// }