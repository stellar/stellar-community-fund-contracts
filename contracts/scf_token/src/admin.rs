use crate::types::DataKey;
use soroban_sdk::{Address, Env};

pub(crate) fn read_admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}

pub(crate) fn write_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, &admin);
}

pub(crate) trait Admin {
    fn transfer_admin(env: Env, new_admin: Address);
}
