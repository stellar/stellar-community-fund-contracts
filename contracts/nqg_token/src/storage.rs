use crate::types::DataKey;
use soroban_sdk::{contracttype, Address, Env};

#[derive(Clone, Debug, Default)]
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

pub(crate) fn read_total_supply(env: &Env) -> TotalSupply {
    env.storage()
        .instance()
        .get(&DataKey::TotalSupply)
        .unwrap_or_default()
}

pub(crate) fn write_total_supply(env: &Env, value: TotalSupply) {
    env.storage().instance().set(&DataKey::TotalSupply, &value);
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
