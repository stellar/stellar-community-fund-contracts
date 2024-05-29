use crate::e2e::common::contract_utils::governance::LayerAggregator;
use nqg_token::{NQGToken, NQGTokenClient};
use soroban_sdk::testutils::Address as AddressTrait;
use soroban_sdk::{Address, Env, Map, I256};

mod governance {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/governance.wasm");
}

pub fn deploy_contract<'a>(
    env: &Env,
    governance_address: &Address,
    admin: &Address,
) -> NQGTokenClient<'a> {
    let nqg_token_address = env.register_contract(None, NQGToken);
    let nqg_token_client = NQGTokenClient::new(env, &nqg_token_address);

    nqg_token_client.initialize(admin, governance_address);

    nqg_token_client
}

pub fn deploy_nqg_contract<'a>(env: &Env, admin: &Address) -> governance::Client<'a> {
    let governance_address = env.register_contract_wasm(None, governance::WASM);
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
    pub client: NQGTokenClient<'a>,
    pub governance_client: governance::Client<'a>,
    pub address: Address,
}

pub fn deploy_and_setup<'a>(env: &Env, admin: &Address) -> Deployment<'a> {
    let governance_client = deploy_nqg_contract(env, admin);
    let client = deploy_contract(env, &governance_client.address, admin);

    let address = Address::generate(&env);

    let mut result = Map::new(&env);
    result.set(address.to_string(), I256::from_i128(&env, 10_i128.pow(18)));

    governance_client.set_neuron_result(
        &soroban_sdk::String::from_str(&env, "0"),
        &soroban_sdk::String::from_str(&env, "0"),
        &result,
    );

    governance_client.calculate_voting_powers();

    env.budget().reset_default();
    client.update_balance(&address);

    Deployment {
        client,
        governance_client,
        address,
    }
}
