use soroban_sdk::{contracterror, contracttype, Address};

pub const DECIMALS: u32 = 18;

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub enum DataKey {
    Admin,
    VotesAdminAddress,
    GovernanceAddress,
    Balance(Address),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[contracterror]
pub enum GovernorWrapperError {
    VotingPowerMissingForUser = 1,
}
