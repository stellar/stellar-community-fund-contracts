#![no_std]

mod storage;

use soroban_sdk::{contract, contracttype, Address, Env, I256, contractimpl};
use crate::storage::{read_admin, write_admin};

mod votes_admin {
    use soroban_sdk::contractimport;

    contractimport!(file = "soroban_admin_votes.wasm");
}

mod governance {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/governance.wasm");
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub enum DataKey {
    Admin,
    VotesAdminAddress,
    GovernanceAddress,
    Balance(Address),
}

#[contract]
pub struct GovernorWrapper;

#[contractimpl]
impl GovernorWrapper {
    pub fn initialize(
        env: Env,
        admin: Address,
        votes_admin_address: Address,
        governance_address: Address,
    ) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Contract already initialized");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::VotesAdminAddress, &votes_admin_address);
        env.storage()
            .instance()
            .set(&DataKey::GovernanceAddress, &governance_address);
    }

    pub fn transfer_admin(env: Env, new_admin: Address) {
        let admin = read_admin(&env);
        admin.require_auth();

        write_admin(&env, &new_admin);
    }

    pub fn update_balance(env: Env, address: Address) {
        let admin = read_admin(&env);
        admin.require_auth();

        let governance_address = env
            .storage()
            .instance()
            .get(&DataKey::GovernanceAddress)
            .unwrap();
        let governance_client = governance::Client::new(&env, &governance_address);
        let voting_powers = governance_client.get_voting_powers();
        // TODO handle unwrap
        let voting_power = voting_powers.get(address.to_string()).unwrap();

        let votes_admin_address = env
            .storage()
            .instance()
            .get(&DataKey::VotesAdminAddress)
            .unwrap();
        let votes_admin_client = votes_admin::Client::new(&env, &votes_admin_address);

        votes_admin_client.clawback(&address, &votes_admin_client.balance(&address));

        let voting_power_whole = fixed_point_decimal_to_whole(&env, voting_power, 18);
        let voting_power_u96 = convert_i256_to_u96(&env, voting_power_whole);

        votes_admin_client.mint(&address, &voting_power_u96);
    }
}

fn fixed_point_decimal_to_whole(env: &Env, value: I256, decimals: u32) -> I256 {
    value.div(&I256::from_i32(&env, 10).pow(decimals))
}

/// Convert value from i256 to u96
///
/// Note: This doesn't perform any scaling, so if the I256 value is over the u96 range, this
/// will panic
fn convert_i256_to_u96(env: &Env, value: I256) -> i128 {
    let i128_value: i128 = if value.le(&I256::from_i32(&env, 0)) {
        0
    } else {
        value.to_i128().unwrap()
    };

    if i128_value < 0 || i128_value >= 2_i128.pow(96) {
        panic!("Value too large to fit in u96");
    }

    i128_value
}
