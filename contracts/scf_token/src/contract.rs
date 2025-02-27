use crate::admin::{read_admin, write_admin, Admin};
use soroban_sdk::token::Interface;
use soroban_sdk::{
    assert_with_error, contract, contractimpl, panic_with_error, vec, Address, BytesN, Env, String,
    Vec, I256,
};

use crate::balance::{extend_balance, read_balance, write_balance};
use crate::storage::{
    read_all_addresses, read_governance_contract_address, read_total_supply, write_all_addresses,
    write_governance_contract_address, write_total_supply,
};
use crate::types::{ContractError, DataKey, VotesError};
use crate::votes::Votes;

pub const DECIMALS: u32 = 9;
const NQG_DECIMALS: u32 = 18;

mod governance {
    use soroban_sdk::contractimport;

    contractimport!(file = "../target/wasm32-unknown-unknown/release/governance.wasm");
}

#[contract]
pub struct SCFToken;

#[contractimpl]
#[allow(clippy::needless_pass_by_value)]
impl SCFToken {
    pub fn initialize(env: Env, admin: Address, governance_address: Address) {
        assert_with_error!(
            env,
            !env.storage().instance().has(&DataKey::Admin),
            ContractError::ContractAlreadyInitialized
        );

        write_admin(&env, &admin);
        write_governance_contract_address(&env, &governance_address);
    }

    pub fn initialize_manual(env: Env, admin: Address) {
        assert_with_error!(
            env,
            !env.storage().instance().has(&DataKey::Admin),
            ContractError::ContractAlreadyInitialized
        );

        write_admin(&env, &admin);
    }

    pub fn update_balance(env: Env, address: Address) -> Result<(), ContractError> {
        let admin = read_admin(&env);
        admin.require_auth();

        let governance_address = read_governance_contract_address(&env);
        let governance_client = governance::Client::new(&env, &governance_address);

        let current_ledger = env.ledger().sequence();
        let current_round = governance_client.get_current_round();

        let old_balance = read_balance(&env, &address);

        assert_with_error!(
            env,
            old_balance.updated_round < current_round,
            ContractError::VotingPowerAlreadyUpdatedForUser
        );

        let voting_power = voting_power_for_user(&env, &governance_client, &address)?;

        let voting_power_whole = scf_score_to_balance(&env, &voting_power);
        let voting_power_i128: i128 = voting_power_whole
            .to_i128()
            .expect("Failed to convert voting power to i128");

        let new_balance = old_balance.new_balance(voting_power_i128, current_ledger, current_round);

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

        let mut addresses = read_all_addresses(&env);
        addresses.push_back(address);
        write_all_addresses(&env, &addresses);

        Ok(())
    }

    pub fn update_balance_manual(
        env: Env,
        address: Address,
        value: I256,
        current_round: u32,
    ) -> Result<(), ContractError> {
        let admin = read_admin(&env);
        admin.require_auth();

        let current_ledger = env.ledger().sequence();

        let old_balance = read_balance(&env, &address);

        assert_with_error!(
            env,
            old_balance.updated_round < current_round,
            ContractError::VotingPowerAlreadyUpdatedForUser
        );

        let voting_power = value;

        let voting_power_whole = scf_score_to_balance(&env, &voting_power);
        let voting_power_i128: i128 = voting_power_whole
            .to_i128()
            .expect("Failed to convert voting power to i128");

        let new_balance = old_balance.new_balance(voting_power_i128, current_ledger, current_round);

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

        let mut addresses = read_all_addresses(&env);
        addresses.push_back(address);
        write_all_addresses(&env, &addresses);

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

    pub fn optimal_threshold(env: Env) -> i128 {
        let admin = read_admin(&env);
        admin.require_auth();

        let user_base_target_percent: u32 = 10; // what top percentage of users should be able to create proposals
        let minimal_user_base_count: u32 = 5; // how many users minimum can create proposals

        let addresses = read_all_addresses(&env);
        if addresses.len() == 0 {
            panic_with_error!(env, ContractError::ZeroUserCount)
        }

        let mut balances_sorted: Vec<i128> = vec![&env];
        for address in addresses {
            let balance = read_balance(&env, &address).current;
            insert_sorted(&mut balances_sorted, balance);
        }

        let target_n: u32 =
            ((balances_sorted.len() * user_base_target_percent) / 100).max(minimal_user_base_count);

        if target_n > balances_sorted.len() {
            return balances_sorted.get(0).unwrap();
        }
        balances_sorted
            .get(balances_sorted.len() - target_n)
            .unwrap()
    }
    pub fn all_addresses(env: Env) -> Vec<Address> {
        let admin = read_admin(&env);
        admin.require_auth();

        read_all_addresses(&env)
    }
    pub fn balance_round(env: Env, address: Address) -> u32 {
        read_balance(&env, &address).updated_round
    }
}

fn insert_sorted(vec: &mut Vec<i128>, value: i128) {
    match vec.iter().position(|x| x > value) {
        Some(pos) => vec.insert(pos as u32, value),
        None => vec.push_back(value),
    };
}

fn voting_power_for_user(
    env: &Env,
    governance_client: &governance::Client,
    address: &Address,
) -> Result<I256, ContractError> {
    let voting_powers = governance_client.get_voting_powers();
    let voting_powers = voting_powers
        .get(address.to_string())
        .ok_or(ContractError::VotingPowerMissingForUser)?;
    Ok(if voting_powers >= I256::from_i32(env, 0) {
        voting_powers
    } else {
        I256::from_i32(env, 0)
    })
}

fn scf_score_to_balance(env: &Env, value: &I256) -> I256 {
    let decimal_shift = NQG_DECIMALS - DECIMALS;
    value.div(&I256::from_i32(env, 10).pow(decimal_shift))
}

#[contractimpl]
#[allow(unused_variables)]
impl Votes for SCFToken {
    fn total_supply(e: Env) -> i128 {
        read_total_supply(&e).current
    }

    fn set_vote_sequence(e: Env, sequence: u32) {}

    fn get_past_total_supply(e: Env, sequence: u32) -> i128 {
        assert_with_error!(
            e,
            sequence < e.ledger().sequence(),
            VotesError::SequenceGreaterThanCurrent
        );

        let total_supply = read_total_supply(&e);
        // It should be safe to store only one checkpoint for total_supply as NQG balances are
        // updated roughly every month. This is assumed to be a much longer period that the voting
        // period and delay will be for the related governor contract.
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
        assert_with_error!(
            e,
            sequence < e.ledger().sequence(),
            VotesError::SequenceGreaterThanCurrent
        );

        let balance = read_balance(&e, &user);
        // It should be safe to store only one checkpoint for balance as NQG balances are
        // updated roughly every month. This is assumed to be a much longer period that the voting
        // period and delay will be for the related governor contract.
        if balance.updated_ledger > sequence {
            balance.previous
        } else {
            balance.current
        }
    }

    fn get_delegate(e: Env, account: Address) -> Address {
        account
    }

    fn delegate(e: Env, account: Address, delegatee: Address) {
        panic_with_error!(e, VotesError::ActionNotSupported)
    }
}

#[contractimpl]
impl Admin for SCFToken {
    fn transfer_admin(env: Env, new_admin: Address) {
        let admin = read_admin(&env);
        admin.require_auth();

        write_admin(&env, &new_admin);
    }
}

#[allow(unused_variables)]
#[contractimpl]
impl Interface for SCFToken {
    fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        panic_with_error!(env, ContractError::ActionNotSupported);
    }

    fn approve(env: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        panic_with_error!(env, ContractError::ActionNotSupported);
    }

    fn balance(env: Env, id: Address) -> i128 {
        read_balance(&env, &id).current
    }

    fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        panic_with_error!(env, ContractError::ActionNotSupported);
    }

    fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        panic_with_error!(env, ContractError::ActionNotSupported);
    }

    fn burn(env: Env, from: Address, amount: i128) {
        panic_with_error!(env, ContractError::ActionNotSupported);
    }

    fn burn_from(env: Env, spender: Address, from: Address, amount: i128) {
        panic_with_error!(env, ContractError::ActionNotSupported);
    }

    fn decimals(env: Env) -> u32 {
        DECIMALS
    }

    fn name(env: Env) -> String {
        String::from_str(&env, "SCF")
    }

    fn symbol(env: Env) -> String {
        String::from_str(&env, "SCF")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converting_scf_values() {
        let env = Env::default();

        let base_value = I256::from_i32(&env, 1);
        let scf_value = base_value.mul(&I256::from_i32(&env, 10).pow(NQG_DECIMALS));
        assert_eq!(
            scf_score_to_balance(&env, &scf_value),
            base_value.mul(&I256::from_i32(&env, 10).pow(DECIMALS))
        );

        let base_value = I256::from_i32(&env, 0);
        let scf_value = base_value.mul(&I256::from_i32(&env, 10).pow(NQG_DECIMALS));
        assert_eq!(
            scf_score_to_balance(&env, &scf_value),
            base_value.mul(&I256::from_i32(&env, 10).pow(DECIMALS))
        );

        let base_value = I256::from_i128(&env, 2_i128.pow(100));
        let scf_value = base_value.mul(&I256::from_i32(&env, 10).pow(NQG_DECIMALS));
        assert_eq!(
            scf_score_to_balance(&env, &scf_value),
            base_value.mul(&I256::from_i32(&env, 10).pow(DECIMALS))
        );

        let base_value = I256::from_i128(&env, 2_i128.pow(100));
        let scf_value = base_value.mul(&I256::from_i32(&env, 10).pow(NQG_DECIMALS));
        assert_eq!(
            scf_score_to_balance(&env, &scf_value),
            base_value.mul(&I256::from_i32(&env, 10).pow(DECIMALS))
        );

        let scf_value = I256::from_i128(&env, 123_456_789_112_233_445);
        assert_eq!(
            scf_score_to_balance(&env, &scf_value),
            I256::from_i128(&env, 123_456_789)
        );
    }
}
