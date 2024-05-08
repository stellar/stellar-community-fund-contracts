use governance::{VotingSystem, VotingSystemClient};
use soroban_sdk::testutils::Address as AddressTrait;
use soroban_sdk::{Address, Env};

pub fn deploy_contract_without_initialization(env: &Env) -> VotingSystemClient {
    let contract_id = env.register_contract(None, VotingSystem);
    let contract_client = VotingSystemClient::new(env, &contract_id);

    contract_client
}

pub fn deploy_contract(env: &Env) -> VotingSystemClient {
    let contract_client = deploy_contract_without_initialization(env);

    env.mock_all_auths();
    let admin = Address::generate(env);
    contract_client.initialize(&admin, &25);

    contract_client
}
