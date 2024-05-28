use crate::types::DataKey;
use soroban_sdk::{Address, Env};

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
