use std::collections::HashMap;

pub mod assigned_reputation;
pub mod prior_voting_history;
pub mod trust_graph;

pub trait Neuron {
    fn name(&self) -> String;

    fn calculate_result(&self, users: &[String]) -> HashMap<String, f64>;
}
