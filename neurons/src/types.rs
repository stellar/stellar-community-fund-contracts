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

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-9;

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < EPS, "expected {expected}, got {actual}");
    }

    #[test]
    fn midpoint_value() {
        // at x = x_off the exp term is 1, so value == a + (k-a)/(c+q)^(1/nu).
        // retro set reduces to 5*tanh(0.2*0) = 0
        assert_close(generalised_logistic_function(-5.0, 5.0, 1.0, 1.0, 0.4, 1.0, 0.0, 0.0), 0.0);
        // prior per-round set at its midpoint: 0 + 1/(1+1)^(1/4)
        let expected = 1.0 / 2.0_f64.powf(1.0 / 4.0);
        assert_close(
            generalised_logistic_function(0.0, 1.0, 1.0, 1.0, 1.0, 4.0, 30.0, 30.0),
            expected,
        );
    }

    #[test]
    fn asymptotes() {
        // every production param set uses c = 1, so x -> +inf gives k and x -> -inf gives a.
        let big = 1e6;
        assert_close(
            generalised_logistic_function(30.0, 100.0, 1.0, 1.0, 0.2, 3.0, 70.0, big),
            100.0,
        );
        assert_close(
            generalised_logistic_function(30.0, 100.0, 1.0, 1.0, 0.2, 3.0, 70.0, -big),
            30.0,
        );
    }

    #[test]
    fn monotonic_increasing() {
        // b > 0 => strictly increasing in x
        let f = |x: f64| generalised_logistic_function(0.0, 1.0, 1.0, 1.0, 1.0, 4.0, 30.0, x);
        let mut prev = f(-10.0);
        let mut x = -9.0;
        while x <= 50.0 {
            let cur = f(x);
            assert!(cur > prev, "expected increasing at x={x}: {cur} !> {prev}");
            prev = cur;
            x += 1.0;
        }
    }

    #[test]
    fn matches_known_param_sets() {
        // retro set is exactly 5*tanh(0.2*x)
        let retro = |x: f64| generalised_logistic_function(-5.0, 5.0, 1.0, 1.0, 0.4, 1.0, 0.0, x);
        assert_close(retro(1.0), 5.0 * 0.2_f64.tanh());
        assert_close(retro(-2.0), 5.0 * (-0.4_f64).tanh());
        // trust set at its midpoint (x_off = 70): 30 + 70/(2)^(1/3)
        let expected = 30.0 + 70.0 / 2.0_f64.powf(1.0 / 3.0);
        assert_close(
            generalised_logistic_function(30.0, 100.0, 1.0, 1.0, 0.2, 3.0, 70.0, 70.0),
            expected,
        );
    }
}
