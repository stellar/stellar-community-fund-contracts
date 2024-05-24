use soroban_sdk::{contracttype, Address};

pub const DECIMALS: u32 = 18;

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub enum DataKey {
    Admin,
    VotesAdminAddress,
    GovernanceAddress,
    Balance(Address),
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub enum GovernorWrapperError {
    VotingPowerMissingForUser = 1,
}
