use soroban_sdk::{contracterror, contracttype, Address};

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub enum DataKey {
    Admin,
    GovernanceAddress,
    Balance(Address),
    TotalSupply,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[contracterror]
pub enum GovernorWrapperError {
    VotingPowerMissingForUser = 1,
}
