use governance::{VotingSystem, VotingSystemClient};
use soroban_sdk::testutils::Address as AddressTrait;
use soroban_sdk::{Address, Env};

pub fn deploy_contract(env: &Env) -> (VotingSystemClient, Address) {
    let admin = Address::generate(env);
    let contract_id = env.register(VotingSystem, (admin.clone(), 25u32));
    let contract_client = VotingSystemClient::new(env, &contract_id);

    (contract_client, admin)
}
