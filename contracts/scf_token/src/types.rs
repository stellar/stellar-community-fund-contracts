use soroban_sdk::{contracterror, contracttype, Address};

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub enum DataKey {
    Admin,
    GovernanceAddress,
    Balance(Address),
    TotalSupply,
    Addresses,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum ContractError {
    VotingPowerMissingForUser = 1,
    ContractAlreadyInitialized = 2,
    VotingPowerAlreadyUpdatedForUser = 3,
    ActionNotSupported = 4,
    ZeroUserCount = 5,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum VotesError {
    ActionNotSupported = 100,
    SequenceGreaterThanCurrent = 101,
}
