use serde::{Deserialize, Serialize};

pub mod neurons;
pub mod quorum;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum Vote {
    Yes,
    No,
    Delegate,
    Abstain,
}
