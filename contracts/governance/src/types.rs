use soroban_sdk::{contracterror, contracttype, String};

pub const ABSTAIN_VOTING_POWER: i32 = 0;

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SubmissionCategory {
    Applications,
    FinancialProtocols,
    InfrastructureAndServices,
    DeveloperTooling,
}

#[contracttype]
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Submission {
    pub id: String,
    pub category: SubmissionCategory,
}

impl Submission {
    pub fn new(id: String, category: SubmissionCategory) -> Self {
        Self { id, category }
    }
}

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
    TallyResultsNotSet = 16,
}
