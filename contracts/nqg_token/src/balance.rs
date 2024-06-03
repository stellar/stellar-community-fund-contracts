use crate::DataKey;
use soroban_sdk::{Address, Env};

const DAY_IN_LEDGERS: u32 = 17280;
const BALANCE_BUMP_VALUE: u32 = 90 * DAY_IN_LEDGERS;
const BALANCE_BUMP_THRESHOLD: u32 = 45 * DAY_IN_LEDGERS;

pub(crate) fn read_balance(env: &Env, address: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Balance(address.clone()))
        .unwrap_or(0)
}

pub(crate) fn write_balance(env: &Env, address: &Address, balance: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::Balance(address.clone()), &balance);
}

pub(crate) fn extend_balance(env: &Env, address: &Address) {
    env.storage().persistent().extend_ttl(
        &DataKey::Balance(address.clone()),
        BALANCE_BUMP_THRESHOLD,
        BALANCE_BUMP_VALUE,
    );
}
