use crate::{
    balance::{BALANCE_BUMP_THRESHOLD, BALANCE_BUMP_VALUE},
    types::DataKey,
};
use soroban_sdk::{contracttype, vec, Address, Env, Vec};

#[derive(Clone, Debug)]
#[contracttype]
pub(crate) struct TotalSupply {
    pub current: i128,
    pub previous: i128,
    pub updated: u32,
}

impl TotalSupply {
    pub(crate) fn new_total_supply(self, value: i128, updated: u32) -> Self {
        Self {
            current: value,
            previous: self.current,
            updated,
        }
    }
}

pub(crate) fn read_all_addresses(env: &Env) -> Vec<Address> {
    env.storage()
        .persistent()
        .get(&DataKey::Addresses)
        .unwrap_or(vec![&env])
}

pub(crate) fn update_all_addresses(env: &Env, new_address: Address) {
    let mut addresses = read_all_addresses(&env);
    if !addresses.contains(&new_address) {
        addresses.push_back(new_address);
        env.storage()
            .persistent()
            .set(&DataKey::Addresses, &addresses);
    }
    env.storage().persistent().extend_ttl(
        &DataKey::Addresses,
        BALANCE_BUMP_THRESHOLD,
        BALANCE_BUMP_VALUE,
    );
}

pub(crate) fn read_total_supply(env: &Env) -> TotalSupply {
    env.storage()
        .instance()
        .get(&DataKey::TotalSupply)
        .unwrap_or(TotalSupply {
            current: 0,
            previous: 0,
            updated: 0,
        })
}

pub(crate) fn write_total_supply(env: &Env, value: &TotalSupply) {
    env.storage().instance().set(&DataKey::TotalSupply, value);
}

pub(crate) fn read_governance_contract_address(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::GovernanceAddress)
        .unwrap()
}

pub(crate) fn write_governance_contract_address(env: &Env, admin: &Address) {
    env.storage()
        .instance()
        .set(&DataKey::GovernanceAddress, &admin);
}
