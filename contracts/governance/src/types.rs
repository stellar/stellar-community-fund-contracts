use soroban_sdk::{contracterror, contracttype};

pub const ABSTAIN_VOTING_POWER: i32 = 0;

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Vote {
    Yes,
    No,
    Abstain,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum VotingSystemError {
    UnknownError = 0,
    NeuralGovernanceNotSet = 1,
    DelegateesNotFound = 2,
    UnexpectedValue = 3,
    TooManyDelegatees = 4,
    NotEnoughDelegatees = 5,
    UnknownVote = 6,
    NeuronResultNotSet = 7,
    InvalidLayerAggregator = 8,
    LayerMissing = 9,
    NeuronMissing = 10,
    NGQResultForVoterMissing = 11,
    DelegationCalculationFailed = 12,
    VotesForSubmissionNotSet = 13,
    SubmissionDoesNotExist = 14,
    VotingPowersNotSet = 15,
}
