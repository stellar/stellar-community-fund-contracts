#![allow(non_upper_case_globals)]

use soroban_fixed_point_math::SorobanFixedPoint;
use soroban_sdk::{contracttype, Env, Map, String, Vec, I256};

pub mod traits;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LayerAggregator {
    Sum,
    Product,
}

#[contracttype]
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Neuron {
    pub name: String,
    pub weight: I256,
}

impl Neuron {
    pub fn create(name: String, weight: I256) -> Self {
        Self { name, weight }
    }
}

#[contracttype]
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Layer {
    /// Vec of `neuron_id`s
    pub neurons: Vec<String>,
    pub aggregator: LayerAggregator,
}

impl Layer {
    pub fn create(neurons: Vec<String>, aggregator: LayerAggregator) -> Self {
        Self {
            neurons,
            aggregator,
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[contracttype]
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NGQ {
    /// Vec of `layer_id`s
    pub layers: Vec<String>,
}

impl NGQ {
    pub fn new(env: &Env) -> Self {
        Self {
            layers: Vec::new(env),
        }
    }
}

pub(crate) fn aggregate_result(
    env: &Env,
    result: Map<String, Vec<I256>>,
    layer_aggregator: LayerAggregator,
    decimals: I256,
) -> Map<String, I256> {
    let mut aggregated_result = Map::new(env);
    for (user, res) in result {
        let res = match layer_aggregator {
            LayerAggregator::Sum => res.iter().reduce(|acc, e| acc.add(&e)),
            LayerAggregator::Product => res
                .iter()
                .reduce(|acc, e| acc.fixed_mul_floor(env, &e, &decimals)),
        }
        .unwrap_or_else(|| I256::from_i128(env, 0));
        aggregated_result.set(user, res);
    }
    aggregated_result
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::vec;

    #[test]
    fn creating_neuron() {
        let env = Env::default();

        let name = String::from_str(&env, "abc");
        let weight = I256::from_i32(&env, 100);
        let neuron = Neuron::create(name.clone(), weight.clone());
        assert_eq!(neuron, Neuron { name, weight });
    }

    #[test]
    fn creating_layer() {
        let env = Env::default();

        let layer = Layer::create(
            vec![&env, String::from_str(&env, "neuron1")],
            LayerAggregator::Sum,
        );
        assert_eq!(
            layer,
            Layer {
                neurons: vec![&env, String::from_str(&env, "neuron1")],
                aggregator: LayerAggregator::Sum
            }
        );
    }

    #[test]
    fn creating_ngq() {
        let env = Env::default();

        let ngq = NGQ::new(&env);
        assert_eq!(ngq, NGQ { layers: vec![&env] });
    }

    #[test]
    fn aggregate_empty() {
        let env = Env::default();

        let user1 = String::from_str(&env, "user1");

        let mut result: Map<String, Vec<I256>> = Map::new(&env);
        result.set(user1.clone(), vec![&env]);

        let aggregated = aggregate_result(
            &env,
            result,
            LayerAggregator::Product,
            I256::from_i128(&env, 1),
        );
        assert_eq!(aggregated.get(user1).unwrap(), I256::from_i32(&env, 0));
    }

    #[test]
    fn aggregate_sum() {
        let env = Env::default();

        let user1 = String::from_str(&env, "user1");
        let user2 = String::from_str(&env, "user2");

        let mut result: Map<String, Vec<I256>> = Map::new(&env);
        result.set(
            user1.clone(),
            vec![&env, I256::from_i128(&env, 1), I256::from_i128(&env, 2)],
        );
        result.set(
            user2.clone(),
            vec![&env, I256::from_i128(&env, 3), I256::from_i128(&env, 4)],
        );

        let aggregated =
            aggregate_result(&env, result, LayerAggregator::Sum, I256::from_i128(&env, 1));
        assert_eq!(
            aggregated.get(user1.clone()).unwrap(),
            I256::from_i128(&env, 3)
        );
        assert_eq!(
            aggregated.get(user2.clone()).unwrap(),
            I256::from_i128(&env, 7)
        );
    }

    #[test]
    fn aggregate_product() {
        let env = Env::default();

        let user1 = String::from_str(&env, "user1");
        let user2 = String::from_str(&env, "user2");

        let mut result: Map<String, Vec<I256>> = Map::new(&env);
        result.set(
            user1.clone(),
            vec![&env, I256::from_i128(&env, 1), I256::from_i128(&env, 2)],
        );
        result.set(
            user2.clone(),
            vec![&env, I256::from_i128(&env, 3), I256::from_i128(&env, 4)],
        );

        let aggregated = aggregate_result(
            &env,
            result,
            LayerAggregator::Product,
            I256::from_i128(&env, 1),
        );
        assert_eq!(
            aggregated.get(user1.clone()).unwrap(),
            I256::from_i128(&env, 2)
        );
        assert_eq!(
            aggregated.get(user2.clone()).unwrap(),
            I256::from_i128(&env, 12)
        );
    }
}
