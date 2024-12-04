use serde::{Deserialize, Serialize};

pub mod neurons;
pub mod quorum;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum SubmissionCategory {
    Applications,
    FinancialProtocols,
    InfrastructureAndServices,
    DeveloperTooling,
}

#[non_exhaustive]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Submission {
    pub name: String,
    pub category: SubmissionCategory,
}

impl Submission {
    #[must_use]
    pub fn new(name: String, category: SubmissionCategory) -> Self {
        Self { name, category }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum Vote {
    Yes,
    No,
    Delegate,
    Abstain,
}
