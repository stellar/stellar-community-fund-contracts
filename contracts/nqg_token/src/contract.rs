use crate::admin::{read_admin, write_admin, Admin};
use soroban_sdk::token::Interface;
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, I256};

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

    pub fn set_governance_contract_address(env: Env, governance_address: Address) {
        let admin = read_admin(&env);
        admin.require_auth();

        write_governance_contract_address(&env, &governance_address);
    }

    pub fn upgrade(env: Env, wasm_hash: BytesN<32>) {
        let admin = read_admin(&env);
        admin.require_auth();

        env.deployer().update_current_contract_wasm(wasm_hash);
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

#[contractimpl]
impl Admin for NQGToken {
    fn transfer_admin(env: Env, new_admin: Address) {
        let admin = read_admin(&env);
        admin.require_auth();

        write_admin(&env, &new_admin);
    }
}

#[allow(unused_variables)]
#[contractimpl]
impl Interface for NQGToken {
    fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        panic!("Transfers are not supported")
    }

    fn approve(env: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        panic!("Transfers are not supported")
    }

    fn balance(env: Env, id: Address) -> i128 {
        read_balance(&env, &id)
    }

    fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        panic!("Transfers are not supported")
    }

    fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        panic!("Transfers are not supported")
    }

    fn burn(env: Env, from: Address, amount: i128) {
        panic!("Burning is not supported")
    }

    fn burn_from(env: Env, spender: Address, from: Address, amount: i128) {
        panic!("Burning is not supported")
    }

    fn decimals(env: Env) -> u32 {
        0
    }

    fn name(env: Env) -> soroban_sdk::String {
        String::from_str(&env, "NQG Token")
    }

    fn symbol(env: Env) -> soroban_sdk::String {
        String::from_str(&env, "NQG")
    }
}
