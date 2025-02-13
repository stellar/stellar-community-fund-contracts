use soroban_sdk::testutils::Address as AddressTrait;
use soroban_sdk::{Address, Env};

use crate::e2e::common::contract_utils::{deploy_and_setup, update_balance, Deployment};

#[test]
#[allow(clippy::too_many_lines)]
fn nth_top_balance() {
    let env = Env::default();
    env.budget().reset_unlimited();

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

    let total: i128 = random_balances.iter().sum();
    assert_eq!(client.total_supply(), total * 10_i128.pow(9));
    assert_eq!(client.nth_top_balance(&3), 16 * 10_i128.pow(9));
}