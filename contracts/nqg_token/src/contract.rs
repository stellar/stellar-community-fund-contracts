use crate::admin::{read_admin, write_admin, Admin};
use soroban_sdk::token::Interface;
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, I256};

use crate::balance::{extend_balance, read_balance, write_balance};
use crate::storage::{
    read_governance_contract_address, read_total_supply, write_governance_contract_address,
    write_total_supply,
};
use crate::types::{DataKey, GovernorWrapperError};
use crate::votes::Votes;

pub const DECIMALS: u32 = 9;
const NQG_DECIMALS: u32 = 18;

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

        let voting_power_whole = nqg_score_to_balance(&env, &voting_power);
        let voting_power_i128: i128 = voting_power_whole
            .to_i128()
            .expect("Failed to convert voting power to i128");

        let current_ledger = env.ledger().sequence();

        let old_balance = read_balance(&env, &address);
        let new_balance = old_balance.new_balance(voting_power_i128, current_ledger);

        let balance_change = new_balance.current - new_balance.previous;

        let old_total_supply = read_total_supply(&env);

        let new_total_supply_value = old_total_supply.current + balance_change;
        let new_total_supply_value = if new_total_supply_value >= 0 {
            new_total_supply_value
        } else {
            0
        };
        let new_total_supply = old_total_supply
            .clone()
            .new_total_supply(new_total_supply_value, current_ledger);

        write_total_supply(&env, &new_total_supply);
        write_balance(&env, &address, &new_balance);
        extend_balance(&env, &address);

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

fn nqg_score_to_balance(env: &Env, value: &I256) -> I256 {
    let decimal_shift = NQG_DECIMALS - DECIMALS;
    value.div(&I256::from_i32(env, 10).pow(decimal_shift))
}

#[contractimpl]
#[allow(unused_variables)]
impl Votes for NQGToken {
    fn total_supply(e: Env) -> i128 {
        read_total_supply(&e).current
    }

    fn set_vote_sequence(e: Env, sequence: u32) {}

    fn get_past_total_supply(e: Env, sequence: u32) -> i128 {
        assert!(
            sequence < e.ledger().sequence(),
            "Provided sequence must be lower than current ledger sequence"
        );

        let total_supply = read_total_supply(&e);
        if total_supply.updated > sequence {
            total_supply.previous
        } else {
            total_supply.current
        }
    }

    fn get_votes(e: Env, account: Address) -> i128 {
        read_balance(&e, &account).current
    }

    fn get_past_votes(e: Env, user: Address, sequence: u32) -> i128 {
        assert!(
            sequence < e.ledger().sequence(),
            "Provided sequence must be lower than current ledger sequence"
        );

        let balance = read_balance(&e, &user);
        if balance.updated > sequence {
            balance.previous
        } else {
            balance.current
        }
    }

    fn get_delegate(e: Env, account: Address) -> Address {
        panic!("Delegation is not supported")
    }

    fn delegate(e: Env, account: Address, delegatee: Address) {
        panic!("Delegation is not supported")
    }
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
        read_balance(&env, &id).current
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
        DECIMALS
    }

    fn name(env: Env) -> String {
        String::from_str(&env, "NQG Token")
    }

    fn symbol(env: Env) -> String {
        String::from_str(&env, "NQG")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converting_nqg_values() {
        let env = Env::default();

        let base_value = I256::from_i32(&env, 1);
        let nqg_value = base_value.mul(&I256::from_i32(&env, 10).pow(NQG_DECIMALS));
        assert_eq!(
            nqg_score_to_balance(&env, &nqg_value),
            base_value.mul(&I256::from_i32(&env, 10).pow(DECIMALS))
        );

        let base_value = I256::from_i32(&env, 0);
        let nqg_value = base_value.mul(&I256::from_i32(&env, 10).pow(NQG_DECIMALS));
        assert_eq!(
            nqg_score_to_balance(&env, &nqg_value),
            base_value.mul(&I256::from_i32(&env, 10).pow(DECIMALS))
        );

        let base_value = I256::from_i128(&env, 2_i128.pow(100));
        let nqg_value = base_value.mul(&I256::from_i32(&env, 10).pow(NQG_DECIMALS));
        assert_eq!(
            nqg_score_to_balance(&env, &nqg_value),
            base_value.mul(&I256::from_i32(&env, 10).pow(DECIMALS))
        );

        let base_value = I256::from_i128(&env, 2_i128.pow(100));
        let nqg_value = base_value.mul(&I256::from_i32(&env, 10).pow(NQG_DECIMALS));
        assert_eq!(
            nqg_score_to_balance(&env, &nqg_value),
            base_value.mul(&I256::from_i32(&env, 10).pow(DECIMALS))
        );

        let base_value = I256::from_i128(&env, 1_123_456_789_123_456_789);
        let nqg_value = base_value.mul(&I256::from_i32(&env, 10).pow(NQG_DECIMALS));
        assert_eq!(
            nqg_score_to_balance(&env, &nqg_value),
            base_value.mul(&I256::from_i32(&env, 10).pow(DECIMALS))
        );
    }
}
