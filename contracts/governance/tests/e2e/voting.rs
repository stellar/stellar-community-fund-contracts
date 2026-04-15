use soroban_sdk::testutils::{Address as AddressTrait, MockAuth, MockAuthInvoke};
use soroban_sdk::{vec, Address, Env, IntoVal, Map, String, Vec, I256};

use governance::types::{Vote, VotingSystemError};
use governance::{LayerAggregator, DECIMALS};

use crate::e2e::common::contract_utils::{deploy_contract, deploy_contract_without_initialization};

#[allow(clippy::identity_op)]
#[test]
fn voting_data_upload() {
    let env = Env::default();
    let contract_client = deploy_contract(&env);
    env.cost_estimate().budget().reset_unlimited();

    let mut raw_neurons: Vec<(String, I256)> = Vec::new(&env);
    raw_neurons.push_back((
        String::from_str(&env, "Dummy"),
        I256::from_i128(&env, 2 * DECIMALS),
    ));
    raw_neurons.push_back((
        String::from_str(&env, "TrustGraph"),
        I256::from_i128(&env, 1 * DECIMALS),
    ));
    contract_client.add_layer(&raw_neurons, &LayerAggregator::Sum);

    let user1 = String::from_str(&env, "user1");
    let user2 = String::from_str(&env, "user2");
    let user3 = String::from_str(&env, "user3");
    let submission1 = String::from_str(&env, "submission1");
    let submission2 = String::from_str(&env, "submission2");

    contract_client.set_submissions(&vec![
        &env,
        (submission1.clone(), String::from_str(&env, "Applications")),
        (submission2.clone(), String::from_str(&env, "Applications")),
    ]);

    let mut votes_submission1 = Map::new(&env);
    votes_submission1.set(user1.clone(), Vote::Yes);
    votes_submission1.set(user2.clone(), Vote::Yes);
    votes_submission1.set(user3.clone(), Vote::Yes);

    // TODO use different votes here
    let mut votes_submission2 = Map::new(&env);
    votes_submission2.set(user1.clone(), Vote::Yes);
    votes_submission2.set(user2.clone(), Vote::No);
    votes_submission2.set(user3.clone(), Vote::Abstain);

    contract_client.set_votes_for_submission(&submission1, &votes_submission1);
    contract_client.set_votes_for_submission(&submission2, &votes_submission2);

    contract_client.set_submissions(&vec![
        &env,
        (submission1.clone(), String::from_str(&env, "Applications")),
        (submission2.clone(), String::from_str(&env, "Applications")),
    ]);

    let mut neuron_result = Map::new(&env);
    neuron_result.set(user1.clone(), I256::from_i128(&env, 100 * DECIMALS));
    neuron_result.set(user2.clone(), I256::from_i128(&env, 200 * DECIMALS));
    neuron_result.set(user3.clone(), I256::from_i128(&env, 300 * DECIMALS));

    let mut neuron_result2 = Map::new(&env);
    neuron_result2.set(user1.clone(), I256::from_i128(&env, 1000 * DECIMALS));
    neuron_result2.set(user2.clone(), I256::from_i128(&env, 2000 * DECIMALS));
    neuron_result2.set(user3.clone(), I256::from_i128(&env, 3000 * DECIMALS));

    contract_client.set_neuron_result(
        &String::from_str(&env, "0"),
        &String::from_str(&env, "0"),
        &neuron_result,
    );
    contract_client.set_neuron_result(
        &String::from_str(&env, "0"),
        &String::from_str(&env, "1"),
        &neuron_result2,
    );

    env.cost_estimate().budget().reset_default();
    contract_client.calculate_voting_powers();
    let result = contract_client.tally_submission(&submission1);
    println!("{}", env.cost_estimate().budget());

    assert_eq!(
        result,
        I256::from_i128(
            &env,
            (100 * 2 + 200 * 2 + 300 * 2 + 1000 + 2000 + 3000) * DECIMALS
        )
    );

    env.cost_estimate().budget().reset_default();
    let result2 = contract_client.tally_submission(&submission2);
    println!("{}", env.cost_estimate().budget());

    assert_eq!(
        result2,
        I256::from_i128(&env, (100 * 2 - 200 * 2 + 1000 - 2000 + 0) * DECIMALS)
    );
}

#[test]
fn setting_votes_for_unknown_submission() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();

    let contract_client = deploy_contract(&env);
    assert_eq!(
        contract_client
            .try_set_votes_for_submission(&String::from_str(&env, "sub1"), &Map::new(&env))
            .unwrap_err()
            .unwrap(),
        VotingSystemError::SubmissionDoesNotExist
    );
}

#[test]
fn tally_submission_requires_admin() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();

    let contract_client = deploy_contract_without_initialization(&env);
    let admin = Address::generate(&env);
    contract_client.initialize(&admin, &25);

    // Set up a submission and votes with admin auth
    let submission = String::from_str(&env, "test_submission");
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &contract_client.address,
            fn_name: "set_submissions",
            args: vec![
                &env,
                vec![
                    &env,
                    (submission.clone(), String::from_str(&env, "Applications")),
                ]
                .to_val(),
            ],
            sub_invokes: &[],
        },
    }]);
    contract_client.set_submissions(&vec![
        &env,
        (submission.clone(), String::from_str(&env, "Applications")),
    ]);

    let user = String::from_str(&env, "user1");
    let mut votes = Map::new(&env);
    votes.set(user.clone(), Vote::Yes);
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &contract_client.address,
            fn_name: "set_votes_for_submission",
            args: vec![&env, submission.clone().to_val(), votes.clone().to_val()],
            sub_invokes: &[],
        },
    }]);
    contract_client.set_votes_for_submission(&submission, &votes);

    // Set up neuron results and calculate voting powers with admin auth
    let mut raw_neurons: Vec<(String, I256)> = Vec::new(&env);
    raw_neurons.push_back((
        String::from_str(&env, "Dummy"),
        I256::from_i128(&env, DECIMALS),
    ));
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &contract_client.address,
            fn_name: "add_layer",
            args: vec![
                &env,
                raw_neurons.clone().to_val(),
                LayerAggregator::Sum.into_val(&env),
            ],
            sub_invokes: &[],
        },
    }]);
    contract_client.add_layer(&raw_neurons, &LayerAggregator::Sum);

    let mut neuron_result = Map::new(&env);
    neuron_result.set(user, I256::from_i128(&env, 100 * DECIMALS));
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &contract_client.address,
            fn_name: "set_neuron_result",
            args: vec![
                &env,
                String::from_str(&env, "0").to_val(),
                String::from_str(&env, "0").to_val(),
                neuron_result.clone().to_val(),
            ],
            sub_invokes: &[],
        },
    }]);
    contract_client.set_neuron_result(
        &String::from_str(&env, "0"),
        &String::from_str(&env, "0"),
        &neuron_result,
    );

    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &contract_client.address,
            fn_name: "calculate_voting_powers",
            args: vec![&env],
            sub_invokes: &[],
        },
    }]);
    contract_client.calculate_voting_powers();

    // Try to tally without admin auth - should fail
    let result = contract_client.try_tally_submission(&submission);
    assert!(result.is_err());
}

#[test]
fn adding_duplicate_submissions() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();

    let contract_client = deploy_contract(&env);

    contract_client.set_submissions(&vec![
        &env,
        (
            String::from_str(&env, "a"),
            String::from_str(&env, "Applications"),
        ),
        (
            String::from_str(&env, "a"),
            String::from_str(&env, "Applications"),
        ),
    ]);

    let submissions = contract_client.get_submissions();
    let mut expected = Vec::new(&env);
    expected.push_back((
        String::from_str(&env, "a"),
        String::from_str(&env, "Applications"),
    ));

    assert_eq!(submissions, expected);
}

#[test]
fn setting_round() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();

    let contract_client = deploy_contract(&env);

    contract_client.set_current_round(&20);
    assert_eq!(contract_client.get_current_round(), 20);

    contract_client.set_current_round(&30);
    assert_eq!(contract_client.get_current_round(), 30);
}

#[test]
fn set_bump_round_flow() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();

    let contract_client = deploy_contract(&env);
    contract_client.set_current_round(&25);

    let submission = String::from_str(&env, "sub1");
    let user1 = String::from_str(&env, "user1");
    let user2 = String::from_str(&env, "user2");
    let neuron0 = String::from_str(&env, "0");
    let layer0 = String::from_str(&env, "0");

    // Setup contract
    contract_client.add_layer(
        &soroban_sdk::vec![
            &env,
            (
                neuron0.clone(),
                I256::from_i128(&env, 1_000_000_000_000_000_000)
            )
        ],
        &LayerAggregator::Sum,
    );

    // Set votes and results for round 25
    contract_client.set_submissions(&vec![
        &env,
        (submission.clone(), String::from_str(&env, "Applications")),
    ]);

    let mut votes25 = Map::new(&env);
    votes25.set(user1.clone(), Vote::Yes);
    votes25.set(user2.clone(), Vote::No);
    contract_client.set_votes_for_submission(&submission, &votes25);
    let expected25 = votes25.clone();

    let mut result25 = Map::new(&env);
    result25.set(user1.clone(), I256::from_i128(&env, 100));
    result25.set(user2.clone(), I256::from_i128(&env, 200));
    contract_client.set_neuron_result(&layer0, &neuron0, &result25);

    // Verify results are set
    assert_eq!(
        contract_client.get_votes_for_submission(&submission),
        expected25
    );
    assert_eq!(
        contract_client.get_neuron_result(&layer0, &neuron0),
        result25
    );

    // Verify submission is active
    assert!(contract_client
        .get_submissions()
        .iter()
        .any(|(name, _category)| name == submission));

    // Bump the round
    contract_client.set_current_round(&26);

    // Verify results are unset for previous round submission
    assert_eq!(
        contract_client
            .try_get_votes_for_submission(&submission)
            .unwrap_err()
            .unwrap(),
        VotingSystemError::VotesForSubmissionNotSet
    );
    assert_eq!(
        contract_client
            .try_get_neuron_result(&layer0, &neuron0)
            .unwrap_err()
            .unwrap(),
        VotingSystemError::NeuronResultNotSet
    );

    // Set votes and results for round 26
    let new_submission = String::from_str(&env, "sub2");
    contract_client.set_submissions(&vec![
        &env,
        (
            new_submission.clone(),
            String::from_str(&env, "Applications"),
        ),
    ]);

    let mut votes26 = Map::new(&env);
    votes26.set(user1.clone(), Vote::No);
    votes26.set(user2.clone(), Vote::Yes);
    contract_client.set_votes_for_submission(&new_submission, &votes26);
    let expected26 = votes26.clone();

    let mut result26 = Map::new(&env);
    result26.set(user1.clone(), I256::from_i128(&env, 5000));
    result26.set(user2.clone(), I256::from_i128(&env, 6000));
    contract_client.set_neuron_result(&layer0, &neuron0, &result26);

    // Verify results are set
    assert_eq!(
        contract_client.get_votes_for_submission(&new_submission),
        expected26
    );
    assert_eq!(
        contract_client.get_neuron_result(&layer0, &neuron0),
        result26
    );

    // Verify new submission is active and old is not
    assert!(contract_client
        .get_submissions()
        .iter()
        .any(|(name, _category)| name == new_submission));
    assert!(!contract_client
        .get_submissions()
        .iter()
        .any(|(name, _category)| name == submission));

    // Verify historical results are still accessible
    assert_eq!(
        contract_client.get_votes_for_submission_round(&submission, &25),
        expected25
    );
    assert_eq!(
        contract_client.get_neuron_result_round(&layer0, &neuron0, &25),
        result25
    );
}

#[test]
fn get_voting_power_for_user() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();

    let contract_client = deploy_contract(&env);
    contract_client.set_current_round(&25);

    let user1 = String::from_str(&env, "user1");
    let user2 = String::from_str(&env, "user2");
    let neuron0 = String::from_str(&env, "0");
    let neuron1 = String::from_str(&env, "1");
    let layer0 = String::from_str(&env, "0");

    // Setup contract
    contract_client.add_layer(
        &soroban_sdk::vec![
            &env,
            (
                neuron0.clone(),
                I256::from_i128(&env, 1_000_000_000_000_000_000)
            ),
            (
                neuron1.clone(),
                I256::from_i128(&env, 1_000_000_000_000_000_000)
            )
        ],
        &LayerAggregator::Sum,
    );

    let mut result0 = Map::new(&env);
    result0.set(user1.clone(), I256::from_i128(&env, 100));
    result0.set(user2.clone(), I256::from_i128(&env, 200));
    contract_client.set_neuron_result(&layer0, &neuron0, &result0);

    let mut result1 = Map::new(&env);
    result1.set(user1.clone(), I256::from_i128(&env, 222));
    result1.set(user2.clone(), I256::from_i128(&env, 333));
    contract_client.set_neuron_result(&layer0, &neuron1, &result1);

    // Verify results are set
    assert_eq!(
        contract_client.get_neuron_result(&layer0, &neuron0),
        result0
    );
    assert_eq!(
        contract_client.get_neuron_result(&layer0, &neuron1),
        result1
    );
    contract_client.calculate_voting_powers();
    // Verify correct voting powers are returned for each user
    assert_eq!(
        contract_client.get_voting_power_for_user(&user1),
        I256::from_i32(&env, 322)
    );
    assert_eq!(
        contract_client.get_voting_power_for_user(&user2),
        I256::from_i32(&env, 533)
    );
    // Verify error is returned for invalid user
    assert_eq!(
        contract_client
            .try_get_voting_power_for_user(&String::from_str(&env, "Random user"))
            .unwrap_err()
            .unwrap(),
        VotingSystemError::NGQResultForVoterMissing
    );
}
