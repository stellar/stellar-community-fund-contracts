use soroban_sdk::{Address, Env};

pub(crate) fn read_balance(env: &Env, address: &Address) -> i128 {
    env.storage().persistent().get(address).unwrap_or(0)
}

pub(crate) fn write_balance(env: &Env, address: &Address, balance: i128) {
    env.storage().persistent().set(address, &balance);
}
