use soroban_sdk::testutils::Address as AddressTrait;
use soroban_sdk::{Address, Env};

use crate::e2e::common::contract_utils::{deploy_and_setup, set_nqg_results, Deployment};

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
