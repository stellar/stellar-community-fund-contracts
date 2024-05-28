use crate::admin::{read_admin, write_admin, Admin};
use soroban_sdk::token::Interface;
use soroban_sdk::{contract, contractimpl, Address, Env, String, I256};

use crate::balance::{read_balance, write_balance};
use crate::storage::{read_governance_contract_address, write_governance_contract_address};
use crate::types::{DataKey, GovernorWrapperError};

mod governance {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/governance.wasm");
}

#[contract]
pub struct NQGToken;

#[contractimpl]
#[allow(clippy::needless_pass_by_value)]
impl NQGToken {
    pub fn initialize(env: Env, admin: Address, governance_address: Address) {
        assert!(
            !env.storage().instance().has(&DataKey::Admin),
            "Contract already initialized"
        );

        write_admin(&env, &admin);
        write_governance_contract_address(&env, &governance_address);
    }

    pub fn update_balance(env: Env, address: Address) -> Result<(), GovernorWrapperError> {
        let admin = read_admin(&env);
        admin.require_auth();

        let voting_power = voting_power_for_user(&env, &address)?;

        let voting_power_whole = fixed_point_decimal_to_whole(&env, &voting_power);
        let voting_power_i128: i128 = voting_power_whole
            .to_i128()
            .expect("Failed to convert voting power to i128");

        write_balance(&env, &address, voting_power_i128);

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
    let decimals = 18;
    value.div(&I256::from_i32(env, 10).pow(decimals))
}

impl Admin for NQGToken {
    fn transfer_admin(env: Env, new_admin: Address) {
        let admin = read_admin(&env);
        admin.require_auth();

        write_admin(&env, &new_admin);
    }
}

#[contractimpl]
impl Interface for NQGToken {
    fn allowance(_env: Env, _from: Address, _spender: Address) -> i128 {
        panic!("Transfers are not supported")
    }

    fn approve(
        _env: Env,
        _from: Address,
        _spender: Address,
        _amount: i128,
        _expiration_ledger: u32,
    ) {
        panic!("Transfers are not supported")
    }

    fn balance(env: Env, id: Address) -> i128 {
        read_balance(&env, &id)
    }

    fn transfer(_env: Env, _from: Address, _to: Address, _amount: i128) {
        panic!("Transfers are not supported")
    }

    fn transfer_from(_env: Env, _spender: Address, _from: Address, _to: Address, _amount: i128) {
        panic!("Transfers are not supported")
    }

    fn burn(_env: Env, _from: Address, _amount: i128) {
        panic!("Burning is not supported")
    }

    fn burn_from(_env: Env, _spender: Address, _from: Address, _amount: i128) {
        panic!("Burning is not supported")
    }

    fn decimals(_env: Env) -> u32 {
        0
    }

    fn name(env: Env) -> soroban_sdk::String {
        String::from_str(&env, "NQG Token")
    }

    fn symbol(env: Env) -> soroban_sdk::String {
        String::from_str(&env, "NQG")
    }
}
