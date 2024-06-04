use crate::e2e::common::contract_utils::{deploy_and_setup, jump, Deployment};
use soroban_sdk::testutils::Address as AddressTrait;
use soroban_sdk::{Address, Env, Map, String, I256};

#[test]
fn old_balance_gets_stored() {
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
        client.get_past_votes(&address, &(env.ledger().sequence() -1)),
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
