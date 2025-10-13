use serde::{Deserialize, Serialize};

pub const DECIMALS: i64 = 1_000_000_000_000_000_000;

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
    pub project_name: String,
}

impl Submission {
    #[must_use]
    pub fn new(name: String, category: SubmissionCategory, project_name: String) -> Self {
        Self {
            name,
            category,
            project_name,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum Vote {
    Yes,
    No,
    Delegate,
    Abstain,
}

pub(crate) fn generalised_logistic_function(
    a: f64,
    k: f64,
    c: f64,
    q: f64,
    b: f64,
    nu: f64,
    x_off: f64,
    x: f64,
) -> f64 {
    a + (k - a) / (f64::powf(c + q * f64::exp(-b * (x - x_off)), 1.0 / nu))
}
