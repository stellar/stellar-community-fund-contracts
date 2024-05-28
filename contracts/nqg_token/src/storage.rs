use crate::types::DataKey;
use soroban_sdk::{Address, Env};

pub(crate) fn read_admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}

pub(crate) fn write_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, &admin);
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
