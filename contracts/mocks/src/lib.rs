#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Env, Map, String, I256};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct NeuronResultKeyData {
    layer_id: String,
    neuron_id: String,
    round: u32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum DataKey {
    NeuronResultKey(NeuronResultKeyData),
}

#[contract]
pub struct MockContract;

#[contractimpl]
impl MockContract {
    pub fn is_mock(_env: &Env) -> bool {
        true
    }

    pub fn get_stored_neuron_result(env: &Env) -> Map<String, I256> {
        let key = DataKey::NeuronResultKey(NeuronResultKeyData {
            layer_id: String::from_str(env, "0"),
            neuron_id: String::from_str(env, "0"),
            round: 25,
        });
        env.storage().temporary().get(&key).unwrap()
    }
}
