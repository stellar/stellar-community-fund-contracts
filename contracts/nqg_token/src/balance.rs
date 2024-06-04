use crate::DataKey;
use soroban_sdk::{contracttype, Address, Env};

const DAY_IN_LEDGERS: u32 = 17280;
const BALANCE_BUMP_VALUE: u32 = 90 * DAY_IN_LEDGERS;
const BALANCE_BUMP_THRESHOLD: u32 = 45 * DAY_IN_LEDGERS;

#[derive(Clone, Debug, Default)]
#[contracttype]
pub(crate) struct Balance {
    pub current: i128,
    pub previous: i128,
    pub updated: u32,
}

impl Balance {
    pub(crate) fn new_balance(self, value: i128, updated: u32) -> Self {
        Self {
            current: value,
            previous: self.current,
            updated,
        }
    }
}

pub(crate) fn read_balance(env: &Env, address: &Address) -> Balance {
    env.storage()
        .persistent()
        .get(&DataKey::Balance(address.clone()))
        .unwrap_or_default()
}

pub(crate) fn write_balance(env: &Env, address: &Address, balance: Balance) {
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
