use std::{collections::HashMap, fs::File, io::BufReader};

use crate::neurons::Neuron;

pub struct Neuron2 {
    data: HashMap<String, f64>,
}
impl Neuron2 {
    pub fn from_json(path: &str) -> Neuron2 {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        let data: HashMap<String, f64> = serde_json::from_reader(reader).unwrap();

        Neuron2 { data }
    }

    fn bonus(input_value: f64) -> f64 {
        input_value * 0.8
    }
}
impl Neuron for Neuron2 {
    fn name(&self) -> String {
        String::from("Neuron2")
    }

    fn calculate_result(&self, users: &[String]) -> HashMap<String, f64> {
        let mut result = HashMap::new();

        for user in users {
            let bonus: f64 = Neuron2::bonus(*self.data.get(user).unwrap());
            result.insert(user.into(), bonus);
        }

        result
    }
}
