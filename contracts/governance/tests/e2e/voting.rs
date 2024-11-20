use soroban_sdk::{vec, Env, Map, String, Vec, I256};

use governance::types::{Submission, SubmissionCategory, Vote, VotingSystemError};
use governance::{LayerAggregator, DECIMALS};

use crate::e2e::common::contract_utils::deploy_contract;

#[allow(clippy::identity_op)]
#[test]
fn voting_data_upload() {
    let env = Env::default();
    let contract_client = deploy_contract(&env);
    env.budget().reset_unlimited();

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

    env.budget().reset_default();
    contract_client.calculate_voting_powers();
    let result = contract_client.tally_submission(&submission1);
    println!("{}", env.budget());

    assert_eq!(
        result,
        I256::from_i128(
            &env,
            (100 * 2 + 200 * 2 + 300 * 2 + 1000 + 2000 + 3000) * DECIMALS
        )
    );

    env.budget().reset_default();
    let result2 = contract_client.tally_submission(&submission2);
    println!("{}", env.budget());

    assert_eq!(
        result2,
        I256::from_i128(&env, (100 * 2 - 200 * 2 + 1000 - 2000 + 0) * DECIMALS)
    );
}

#[test]
fn setting_votes_for_unknown_submission() {
    let env = Env::default();
    env.budget().reset_unlimited();

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
fn adding_duplicate_submissions() {
    let env = Env::default();
    env.budget().reset_unlimited();

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
    env.budget().reset_unlimited();

    let contract_client = deploy_contract(&env);

    contract_client.set_current_round(&20);
    assert_eq!(contract_client.get_current_round(), 20);

    contract_client.set_current_round(&30);
    assert_eq!(contract_client.get_current_round(), 30);
}

#[test]
fn set_bump_round_flow() {
    let env = Env::default();
    env.budget().reset_unlimited();

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
