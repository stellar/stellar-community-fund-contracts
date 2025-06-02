use std::collections::HashMap;

pub trait Neuron {
    fn name(&self) -> String;

    fn calculate_result(&self, users: &[String]) -> HashMap<String, f64>;
}
