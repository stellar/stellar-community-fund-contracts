use crate::DataKey;
use soroban_sdk::{contracttype, Address, Env};

const DAY_IN_LEDGERS: u32 = 17280;
pub const BALANCE_BUMP_VALUE: u32 = 90 * DAY_IN_LEDGERS;
pub const BALANCE_BUMP_THRESHOLD: u32 = 45 * DAY_IN_LEDGERS;

#[derive(Clone, Debug)]
#[contracttype]
pub(crate) struct Balance {
    pub current: i128,
    pub previous: i128,
    pub updated_ledger: u32,
    pub updated_round: u32,
}

impl Balance {
    pub(crate) fn new_balance(self, value: i128, ledger: u32, round: u32) -> Self {
        Self {
            current: value,
            previous: self.current,
            updated_ledger: ledger,
            updated_round: round,
        }
    }
}

pub(crate) fn read_balance(env: &Env, address: &Address) -> Balance {
    env.storage()
        .persistent()
        .get(&DataKey::Balance(address.clone()))
        .unwrap_or(Balance {
            current: 0,
            previous: 0,
            updated_ledger: 0,
            updated_round: 0,
        })
}

pub(crate) fn write_balance(env: &Env, address: &Address, balance: &Balance) {
    env.storage()
        .persistent()
        .set(&DataKey::Balance(address.clone()), balance);
}

pub(crate) fn extend_balance(env: &Env, address: &Address) {
    env.storage().persistent().extend_ttl(
        &DataKey::Balance(address.clone()),
        BALANCE_BUMP_THRESHOLD,
        BALANCE_BUMP_VALUE,
    );
}
