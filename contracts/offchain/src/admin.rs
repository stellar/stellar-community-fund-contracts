use soroban_sdk::{Address, Env};

use crate::DataKey;

pub mod traits;

pub(crate) fn require_admin(env: &Env) {
    let admin = get_admin(env);
    admin.require_auth();
}

pub(crate) fn get_admin(env: &Env) -> Address {
    assert!(is_set_admin(env), "Admin not set");
    env.storage().instance().get(&DataKey::Admin).unwrap()
}

pub(crate) fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub(crate) fn is_set_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}
