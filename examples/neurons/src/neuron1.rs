use std::{collections::HashMap, fs::File, io::BufReader};

use crate::neurons::Neuron;

pub struct Neuron1 {
    data: HashMap<String, f64>,
}
impl Neuron1 {
    pub fn from_json(path: &str) -> Neuron1 {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        let data: HashMap<String, f64> = serde_json::from_reader(reader).unwrap();

        Neuron1 { data }
    }

    fn bonus(input_value: f64) -> f64 {
        input_value * 1.5
    }
}
impl Neuron for Neuron1 {
    fn name(&self) -> String {
        String::from("Neuron1")
    }

    fn calculate_result(&self, users: &[String]) -> HashMap<String, f64> {
        let mut result = HashMap::new();

        for user in users {
            let bonus: f64 = Neuron1::bonus(*self.data.get(user).unwrap());
            result.insert(user.into(), bonus);
        }

        result
    }
}
