use soroban_sdk::{vec, Env, String, Vec, I256};

use offchain::types::VotingSystemError;
use offchain::LayerAggregator;

use crate::e2e::common::contract_utils::deploy_contract;

#[test]
fn add_layer() {
    let env = Env::default();
    let contract_client = deploy_contract(&env);

    let governance = contract_client.get_neural_governance();
    assert_eq!(governance.layers, Vec::new(&env));

    let neurons = vec![
        &env,
        (String::from_str(&env, "aaa"), I256::from_i32(&env, 100)),
        (String::from_str(&env, "b"), I256::from_i32(&env, 2000)),
    ];
    contract_client.add_layer(&neurons, &LayerAggregator::Sum);

    let governance = contract_client.get_neural_governance();
    assert_eq!(governance.layers, vec![&env, String::from_str(&env, "0")]);

    let layer = contract_client.get_layer(&String::from_str(&env, "0"));
    assert_eq!(layer.aggregator, LayerAggregator::Sum);
    assert_eq!(
        layer.neurons,
        vec![
            &env,
            String::from_str(&env, "0"),
            String::from_str(&env, "1")
        ]
    );

    let neuron_0 =
        contract_client.get_neuron(&String::from_str(&env, "0"), &String::from_str(&env, "0"));
    assert_eq!(neuron_0.name, String::from_str(&env, "aaa"));
    assert_eq!(neuron_0.weight, I256::from_i32(&env, 100));

    let neuron_1 =
        contract_client.get_neuron(&String::from_str(&env, "0"), &String::from_str(&env, "1"));
    assert_eq!(neuron_1.name, String::from_str(&env, "b"));
    assert_eq!(neuron_1.weight, I256::from_i32(&env, 2000));
}

#[test]
fn remove_layer() {
    let env = Env::default();
    let contract_client = deploy_contract(&env);

    let governance = contract_client.get_neural_governance();
    assert_eq!(governance.layers, Vec::new(&env));

    let neurons = vec![
        &env,
        (String::from_str(&env, "aaa"), I256::from_i32(&env, 100)),
        (String::from_str(&env, "b"), I256::from_i32(&env, 2000)),
    ];
    contract_client.add_layer(&neurons, &LayerAggregator::Sum);

    let governance = contract_client.get_neural_governance();
    assert_eq!(governance.layers, vec![&env, String::from_str(&env, "0")]);

    contract_client.remove_layer(&String::from_str(&env, "0"));

    let governance = contract_client.get_neural_governance();
    assert_eq!(governance.layers, vec![&env]);

    assert_eq!(
        contract_client.try_get_layer(&String::from_str(&env, "0")),
        Err(Ok(VotingSystemError::LayerMissing))
    );
    assert_eq!(
        contract_client.try_get_neuron(&String::from_str(&env, "0"), &String::from_str(&env, "0")),
        Err(Ok(VotingSystemError::NeuronMissing))
    );
    assert_eq!(
        contract_client.try_get_neuron(&String::from_str(&env, "0"), &String::from_str(&env, "1")),
        Err(Ok(VotingSystemError::NeuronMissing))
    );
}

#[test]
fn update_layer() {
    let env = Env::default();
    let contract_client = deploy_contract(&env);

    let governance = contract_client.get_neural_governance();
    assert_eq!(governance.layers, Vec::new(&env));

    let neurons = vec![
        &env,
        (String::from_str(&env, "aaa"), I256::from_i32(&env, 100)),
        (String::from_str(&env, "b"), I256::from_i32(&env, 2000)),
    ];
    contract_client.add_layer(&neurons, &LayerAggregator::Sum);

    let neurons = vec![
        &env,
        (String::from_str(&env, "cc"), I256::from_i32(&env, 3)),
    ];
    contract_client.update_layer(
        &String::from_str(&env, "0"),
        &neurons,
        &LayerAggregator::Product,
    );

    let layer = contract_client.get_layer(&String::from_str(&env, "0"));
    assert_eq!(layer.neurons, vec![&env, String::from_str(&env, "0")]);
    assert_eq!(layer.aggregator, LayerAggregator::Product);

    let neuron =
        contract_client.get_neuron(&String::from_str(&env, "0"), &String::from_str(&env, "0"));
    assert_eq!(neuron.name, String::from_str(&env, "cc"));
    assert_eq!(neuron.weight, I256::from_i32(&env, 3));

    assert_eq!(
        contract_client.try_get_neuron(&String::from_str(&env, "0"), &String::from_str(&env, "1")),
        Err(Ok(VotingSystemError::NeuronMissing))
    );
}

#[test]
fn add_layer_after_removing() {
    let env = Env::default();
    let contract_client = deploy_contract(&env);

    let governance = contract_client.get_neural_governance();
    assert_eq!(governance.layers, Vec::new(&env));

    let neurons = vec![
        &env,
        (String::from_str(&env, "aaa"), I256::from_i32(&env, 100)),
        (String::from_str(&env, "b"), I256::from_i32(&env, 2000)),
    ];
    contract_client.add_layer(&neurons, &LayerAggregator::Sum);

    contract_client.remove_layer(&String::from_str(&env, "0"));

    let neurons = vec![&env, (String::from_str(&env, "c"), I256::from_i32(&env, 1))];
    contract_client.add_layer(&neurons, &LayerAggregator::Product);

    let governance = contract_client.get_neural_governance();
    assert_eq!(governance.layers, vec![&env, String::from_str(&env, "1")]);

    let layer = contract_client.get_layer(&String::from_str(&env, "1"));
    assert_eq!(layer.aggregator, LayerAggregator::Product);
    assert_eq!(layer.neurons, vec![&env, String::from_str(&env, "0"),]);

    let neuron_0 =
        contract_client.get_neuron(&String::from_str(&env, "1"), &String::from_str(&env, "0"));
    assert_eq!(neuron_0.name, String::from_str(&env, "c"));
    assert_eq!(neuron_0.weight, I256::from_i32(&env, 1));
}
