#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, I256};

use crate::governance::DataKey;
use crate::storage::{
    read_admin, read_governance_contract_address, read_votes_admin_contract_address, write_admin,
    write_governance_contract_address, write_votes_admin_contract_address,
};
use crate::types::{GovernorWrapperError, DECIMALS};

mod storage;
mod types;

mod votes_admin {
    use soroban_sdk::contractimport;

    contractimport!(file = "soroban_admin_votes.wasm");
}

mod governance {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/governance.wasm");
}

#[contract]
pub struct GovernorWrapper;

#[contractimpl]
#[allow(clippy::needless_pass_by_value)]
impl GovernorWrapper {
    pub fn initialize(
        env: Env,
        admin: Address,
        votes_admin_address: Address,
        governance_address: Address,
    ) {
        assert!(
            env.storage().instance().has(&DataKey::Admin),
            "Contract already initialized"
        );

        write_admin(&env, &admin);
        write_votes_admin_contract_address(&env, &votes_admin_address);
        write_governance_contract_address(&env, &governance_address);
    }

    pub fn transfer_admin(env: Env, new_admin: Address) {
        let admin = read_admin(&env);
        admin.require_auth();

        write_admin(&env, &new_admin);
    }

    pub fn update_balance(env: Env, address: Address) -> Result<(), GovernorWrapperError> {
        let admin = read_admin(&env);
        admin.require_auth();

        let voting_power = voting_power_for_user(&env, &address)?;

        let votes_admin_address = read_votes_admin_contract_address(&env);
        let votes_admin_client = votes_admin::Client::new(&env, &votes_admin_address);

        votes_admin_client.clawback(&address, &votes_admin_client.balance(&address));

        let voting_power_whole = fixed_point_decimal_to_whole(&env, &voting_power);
        let voting_power_u96 = convert_i256_to_u96(&env, &voting_power_whole);

        votes_admin_client.mint(&address, &voting_power_u96);

        Ok(())
    }
}

fn voting_power_for_user(env: &Env, address: &Address) -> Result<I256, GovernorWrapperError> {
    let governance_address = read_governance_contract_address(env);
    let governance_client = governance::Client::new(env, &governance_address);
    let voting_powers = governance_client.get_voting_powers();
    voting_powers
        .get(address.to_string())
        .ok_or(GovernorWrapperError::VotingPowerMissingForUser)
}

fn fixed_point_decimal_to_whole(env: &Env, value: &I256) -> I256 {
    value.div(&I256::from_i32(env, 10).pow(DECIMALS))
}

/// Convert value from i256 to u96
///
///
/// # Panics
/// If the I256 value is over the u96 range, this will panic
fn convert_i256_to_u96(env: &Env, value: &I256) -> i128 {
    let i128_value: i128 = if value.le(&I256::from_i32(env, 0)) {
        0
    } else {
        value.to_i128().unwrap()
    };

    assert!(
        i128_value < 0 || i128_value >= 2_i128.pow(96),
        "Value too large to fit in u96"
    );

    i128_value
}
