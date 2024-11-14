#![no_std]
#![allow(clippy::needless_pass_by_value)]

extern crate alloc;

use alloc::string::ToString;

use soroban_fixed_point_math::SorobanFixedPoint;
use soroban_sdk::{
    contract, contractimpl, contracttype, vec, Address, BytesN, Env, Map, String, Vec, I256,
};

use admin::{is_set_admin, require_admin};

use crate::admin::set_admin;
use crate::admin::traits::Admin;
use crate::neural_governance::traits::Governance;
pub use crate::neural_governance::LayerAggregator;
use crate::neural_governance::{aggregate_result, Layer, Neuron, NGQ};
use crate::storage::{
    read_layer, read_neural_governance, read_neuron, read_neuron_result, read_submission_votes,
    read_submissions, read_voting_powers, remove_layer, remove_neuron, write_layer,
    write_neural_governance, write_neuron, write_neuron_result, write_submission_votes,
    write_submissions, write_voting_powers, LayerKeyData, NeuronKeyData, NeuronResultKeyData,
    SubmissionVotesKeyData, SubmissionsKeyData, VotingPowersKeyData,
};
use crate::types::{Vote, VotingSystemError, ABSTAIN_VOTING_POWER};

mod admin;
mod neural_governance;
mod storage;
pub mod types;

pub const DECIMALS: i128 = 1_000_000_000_000_000_000;

#[contract]
pub struct VotingSystem;

type ContractResult<T> = Result<T, VotingSystemError>;

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub enum DataKey {
    /// storage type: instance
    NeuralGovernance,
    /// storage type: instance
    /// Map<String, ()>
    Submissions(SubmissionsKeyData),
    /// storage type: instance
    /// Map<user_id, Vec<user_id>> - users to the vector of users they delegated their votes to
    Delegatees,
    // storage type: instance
    // Map<UserUUID, u32> - users to their delegation rank
    DelegationRanks,
    /// storage type: instance
    /// u32
    CurrentLayerId,
    Admin,
    /// u32
    CurrentRound,
    NeuronKey(NeuronKeyData),
    NeuronResultKey(NeuronResultKeyData),
    LayerKey(LayerKeyData),
    SubmissionVotes(SubmissionVotesKeyData),
    VotingPowers(VotingPowersKeyData),
}

#[contractimpl]
impl VotingSystem {
    /// Initialize the governance contract.
    pub fn initialize(env: Env, admin: Address, current_round: u32) {
        assert!(!is_set_admin(&env), "Admin already set");
        set_admin(&env, &admin);

        let neural_governance = NGQ::new(&env);
        env.storage()
            .instance()
            .set(&DataKey::CurrentRound, &current_round);
        write_neural_governance(&env, neural_governance);
    }

    /// Get the current active round.
    pub fn get_current_round(env: &Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::CurrentRound)
            .unwrap()
    }

    /// Change the active round.
    pub fn set_current_round(env: Env, round: u32) {
        require_admin(&env);

        env.storage().instance().set(&DataKey::CurrentRound, &round);
    }

    /// Set multiple submissions.
    pub fn set_submissions(env: Env, new_submissions_raw: Vec<(String, String)>) {
        let mut new_submissions = vec![&env];
        for (name, category) in new_submissions_raw {
            new_submissions.push_back((name, category));
        }

        require_admin(&env);

        let mut submissions = Vec::new(&env);

        for submission in new_submissions {
            if submissions.contains(submission.clone()) {
                continue;
            }
            submissions.push_back(submission);
        }

        write_submissions(&env, Self::get_current_round(&env), &submissions);
    }

    /// Get submissions for the active round.
    pub fn get_submissions(env: &Env) -> Vec<(String, String)> {
        read_submissions(env, Self::get_current_round(env))
    }

    /// Set votes for a submission.
    pub fn set_votes_for_submission(
        env: &Env,
        submission_id: String,
        votes: Map<String, Vote>,
    ) -> Result<(), VotingSystemError> {
        require_admin(env);

        if !read_submissions(env, Self::get_current_round(env))
            .iter()
            .any(|(name, _category)| name == submission_id)
        {
            return Err(VotingSystemError::SubmissionDoesNotExist);
        }

        // this causes timeout god knows why
        write_submission_votes(env, &submission_id, Self::get_current_round(env), &votes);
        Ok(())
    }

    /// Get votes for the submission for a specific round.
    pub fn get_votes_for_submission_round(
        env: &Env,
        submission_id: String,
        round: u32,
    ) -> Result<Map<String, Vote>, VotingSystemError> {
        read_submission_votes(env, &submission_id, round)
    }

    /// Get votes for the submission for the active round
    pub fn get_votes_for_submission(
        env: &Env,
        submission_id: String,
    ) -> Result<Map<String, Vote>, VotingSystemError> {
        Self::get_votes_for_submission_round(env, submission_id, Self::get_current_round(env))
    }

    /// Compute the final voting power of a submission.
    ///
    /// Requires calling `calculate_voting_powers` first to compute and store voting powers for the round.
    ///
    /// # Panics:
    ///
    /// The function will panic if no voting powers are set for the active round.
    pub fn tally_submission(env: &Env, submission_id: String) -> Result<I256, VotingSystemError> {
        let submission_votes = Self::get_votes_for_submission(env, submission_id.clone())?;
        let mut submission_voting_power_plus = I256::from_i32(env, 0);
        let mut submission_voting_power_minus = I256::from_i32(env, 0);
        let voting_powers = Self::get_voting_powers(env.clone())?;

        for (voter_id, vote) in submission_votes {
            let voting_power = match vote {
                Vote::Abstain => I256::from_i32(env, ABSTAIN_VOTING_POWER),
                _ => voting_powers
                    .get(voter_id)
                    .ok_or(VotingSystemError::NGQResultForVoterMissing)?,
            };
            match vote {
                Vote::Yes => {
                    submission_voting_power_plus = submission_voting_power_plus.add(&voting_power);
                }
                Vote::No => {
                    submission_voting_power_minus =
                        submission_voting_power_minus.add(&voting_power);
                }
                Vote::Abstain => (),
            };
        }

        Ok(submission_voting_power_plus.sub(&submission_voting_power_minus))
    }
}

#[contractimpl]
impl Admin for VotingSystem {
    fn transfer_admin(env: Env, new_admin: Address) {
        require_admin(&env);

        set_admin(&env, &new_admin);
    }

    fn upgrade(env: Env, wasm_hash: BytesN<32>) {
        require_admin(&env);

        env.deployer().update_current_contract_wasm(wasm_hash);
    }
}

#[contractimpl]
impl Governance for VotingSystem {
    fn add_layer(
        env: Env,
        raw_neurons: Vec<(String, I256)>,
        layer_aggregator: LayerAggregator,
    ) -> Result<(), VotingSystemError> {
        require_admin(&env);

        let layer_id = next_layer_id(&env);
        let layer_id = String::from_str(&env, layer_id.to_string().as_str());

        create_or_update_layer(env, layer_id, raw_neurons, layer_aggregator);

        Ok(())
    }

    fn remove_layer(env: Env, layer_id: String) -> Result<(), VotingSystemError> {
        require_admin(&env);

        let mut neural_governance = read_neural_governance(&env).unwrap();
        let index = neural_governance
            .layers
            .iter()
            .position(|id| id == layer_id)
            .ok_or(VotingSystemError::LayerMissing)?;
        let layer = read_layer(&env, &layer_id)?;

        for neuron_id in layer.neurons {
            remove_neuron(&env, &layer_id, &neuron_id);
        }
        remove_layer(&env, &layer_id);

        neural_governance
            .layers
            .remove(u32::try_from(index).unwrap());

        write_neural_governance(&env, neural_governance);

        Ok(())
    }

    fn update_layer(
        env: Env,
        layer_id: String,
        raw_neurons: Vec<(String, I256)>,
        layer_aggregator: LayerAggregator,
    ) -> Result<(), VotingSystemError> {
        require_admin(&env);

        let layer = read_layer(&env, &layer_id)?;

        for neuron_id in layer.neurons {
            remove_neuron(&env, &layer_id, &neuron_id);
        }

        create_or_update_layer(env, layer_id, raw_neurons, layer_aggregator);

        Ok(())
    }

    fn get_layer(env: Env, layer_id: String) -> Result<Layer, VotingSystemError> {
        read_layer(&env, &layer_id)
    }

    fn get_neuron(
        env: Env,
        layer_id: String,
        neuron_id: String,
    ) -> Result<Neuron, VotingSystemError> {
        read_neuron(&env, &layer_id, &neuron_id)
    }

    fn get_neuron_result_round(
        env: &Env,
        layer_id: String,
        neuron_id: String,
        round: u32,
    ) -> Result<Map<String, I256>, VotingSystemError> {
        read_neuron_result(env, &layer_id, &neuron_id, round)
    }

    fn get_neuron_result(
        env: &Env,
        layer_id: String,
        neuron_id: String,
    ) -> Result<Map<String, I256>, VotingSystemError> {
        Self::get_neuron_result_round(env, layer_id, neuron_id, Self::get_current_round(env))
    }

    fn set_neuron_result(env: Env, layer_id: String, neuron_id: String, result: Map<String, I256>) {
        require_admin(&env);

        write_neuron_result(
            &env,
            &layer_id,
            &neuron_id,
            Self::get_current_round(&env),
            &result,
        );
    }

    /// Get a result of a whole layer
    ///
    /// Gets a result of each neuron and aggregates them using a configured aggregator function
    fn get_layer_result(
        env: Env,
        layer_id: String,
    ) -> Result<Map<String, I256>, VotingSystemError> {
        let layer = read_layer(&env, &layer_id)?;
        let mut result: Map<String, Vec<I256>> = Map::new(&env);

        for neuron_id in layer.neurons {
            let neuron_result = Self::get_neuron_result(&env, layer_id.clone(), neuron_id.clone())?;
            let neuron = read_neuron(&env, &layer_id, &neuron_id)?;
            let neuron_result = weigh_neuron_result(&env, &neuron.weight, neuron_result);
            for (user, new) in neuron_result {
                let mut previous = result.get(user.clone()).unwrap_or_else(|| Vec::new(&env));
                previous.push_back(new);
                result.set(user, previous);
            }
        }

        Ok(aggregate_result(
            &env,
            result,
            layer.aggregator,
            I256::from_i128(&env, DECIMALS),
        ))
    }

    fn calculate_voting_powers(env: Env) -> Result<(), VotingSystemError> {
        require_admin(&env);

        let neural_governance = read_neural_governance(&env).unwrap();
        let mut result: Map<String, I256> = Map::new(&env);
        for layer_id in neural_governance.layers {
            let layer_result = VotingSystem::get_layer_result(env.clone(), layer_id)?;
            for (key, value) in layer_result {
                result.set(
                    key.clone(),
                    value.add(&result.get(key).unwrap_or_else(|| I256::from_i32(&env, 0))),
                );
            }
        }

        write_voting_powers(&env, Self::get_current_round(&env), &result);
        Ok(())
    }

    fn get_voting_powers(env: Env) -> Result<Map<String, I256>, VotingSystemError> {
        read_voting_powers(&env, Self::get_current_round(&env))
    }

    /// Get a current neural governance setup
    fn get_neural_governance(env: &Env) -> Result<NGQ, VotingSystemError> {
        read_neural_governance(env)
    }
}

fn create_or_update_layer(
    env: Env,
    layer_id: String,
    raw_neurons: Vec<(String, I256)>,
    layer_aggregator: LayerAggregator,
) {
    let mut neural_governance = read_neural_governance(&env).unwrap();

    let mut neurons = Vec::new(&env);

    for (neuron_id_raw, (name, weight)) in raw_neurons.into_iter().enumerate() {
        let neuron_id = String::from_str(&env, neuron_id_raw.to_string().as_str());

        let neuron_details = Neuron::create(name, weight);
        write_neuron(&env, &layer_id, &neuron_id, &neuron_details);

        neurons.push_back(neuron_id);
    }

    let layer = Layer::create(neurons, layer_aggregator);
    write_layer(&env, &layer_id, &layer);

    neural_governance.layers.push_back(layer_id);
    write_neural_governance(&env, neural_governance);
}

fn weigh_neuron_result(env: &Env, weight: &I256, result: Map<String, I256>) -> Map<String, I256> {
    let mut scaled = Map::new(env);

    for (key, value) in result {
        scaled.set(
            key,
            value.fixed_mul_floor(env, weight, &I256::from_i128(env, DECIMALS)),
        );
    }

    scaled
}

fn next_layer_id(env: &Env) -> u32 {
    let id: u32 = env
        .storage()
        .instance()
        .get(&DataKey::CurrentLayerId)
        .unwrap_or(0);
    env.storage()
        .instance()
        .set(&DataKey::CurrentLayerId, &(id + 1));
    id
}
