use crate::e2e::common::contract_utils::governance::LayerAggregator;
use scf_token::{SCFToken, SCFTokenClient};
use soroban_sdk::testutils::{Ledger, LedgerInfo};
use soroban_sdk::{Address, Env, Map, I256};

pub mod governance {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/governance.wasm");
}

pub fn deploy_contract<'a>(
    env: &Env,
    governance_address: &Address,
    admin: &Address,
) -> SCFTokenClient<'a> {
    let scf_token_address = env.register(SCFToken, ());
    let scf_token_client = SCFTokenClient::new(env, &scf_token_address);

    scf_token_client.initialize(admin, governance_address);

    scf_token_client
}

pub fn deploy_scf_contract<'a>(env: &Env, admin: &Address) -> governance::Client<'a> {
    let governance_address = env.register(governance::WASM, ());
    let governance_client = governance::Client::new(env, &governance_address);

    governance_client.initialize(admin, &25);

    let neurons = soroban_sdk::vec![
        &env,
        (
            soroban_sdk::String::from_str(env, "Layer1"),
            I256::from_i128(env, 10_i128.pow(18)),
        ),
    ];
    governance_client.add_layer(&neurons, &LayerAggregator::Sum);

    governance_client
}

pub struct Deployment<'a> {
    pub client: SCFTokenClient<'a>,
    pub governance_client: governance::Client<'a>,
}

pub fn deploy_and_setup<'a>(env: &Env, admin: &Address) -> Deployment<'a> {
    env.mock_all_auths();

    let governance_client = deploy_scf_contract(env, admin);
    let client = deploy_contract(env, &governance_client.address, admin);

    env.set_auths(&[]);

    Deployment {
        client,
        governance_client,
    }
}

pub fn set_nqg_results(
    env: &Env,
    governance_client: &governance::Client,
    address: &Address,
    new_balance: i128,
) {
    let mut result = governance_client
        .try_get_neuron_result(
            &soroban_sdk::String::from_str(env, "0"),
            &soroban_sdk::String::from_str(env, "0"),
        )
        .unwrap_or_else(|_| {
            let mut map = Map::new(env);
            map.set(address.to_string(), I256::from_i32(env, 0));
            Ok(map)
        })
        .unwrap();
    result.set(address.to_string(), I256::from_i128(env, new_balance));

    governance_client.set_neuron_result(
        &soroban_sdk::String::from_str(env, "0"),
        &soroban_sdk::String::from_str(env, "0"),
        &result,
    );

    governance_client.calculate_voting_powers();
}

pub fn update_balance(
    env: &Env,
    client: &SCFTokenClient,
    governance_client: &governance::Client,
    address: &Address,
    new_balance: i128,
) {
    set_nqg_results(env, governance_client, address, new_balance);

    client.update_balance(address);
}

/// Taken from here <https://github.com/script3/soroban-governor/blob/0a7788905366ff52297f3fcecb4c3a0dc9f55cf5/contracts/tests/src/env.rs#L20/>
pub fn jump(env: &mut Env, ledgers: u32) {
    env.ledger().set(LedgerInfo {
        timestamp: env
            .ledger()
            .timestamp()
            .saturating_add(u64::from(ledgers) * 5),
        protocol_version: 20,
        sequence_number: env.ledger().sequence().saturating_add(ledgers),
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10 * 17280,
        min_persistent_entry_ttl: 10 * 17280,
        max_entry_ttl: 365 * 17280,
    });
}

pub fn bump_round(governance_client: &governance::Client) {
    governance_client.set_current_round(&(governance_client.get_current_round() + 1));
}
