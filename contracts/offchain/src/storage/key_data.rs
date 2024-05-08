use crate::DataKey;
use soroban_sdk::{contracttype, String};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct LayerKeyData {
    layer_id: String,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct NeuronResultKeyData {
    layer_id: String,
    neuron_id: String,
    round: u32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct NeuronKeyData {
    layer_id: String,
    neuron_id: String,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct SubmissionVotesKeyData {
    submission_id: String,
    round: u32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct SubmissionsKeyData {
    round: u32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct VotingPowersKeyData {
    round: u32,
}

pub fn get_layer_key(layer_id: &String) -> DataKey {
    let data = LayerKeyData {
        layer_id: layer_id.clone(),
    };
    DataKey::LayerKey(data)
}

pub fn get_neuron_key(layer_id: &String, neuron_id: &String) -> DataKey {
    let data = NeuronKeyData {
        layer_id: layer_id.clone(),
        neuron_id: neuron_id.clone(),
    };
    DataKey::NeuronKey(data)
}

pub fn get_neuron_result_key(layer_id: &String, neuron_id: &String, round: u32) -> DataKey {
    let data = NeuronResultKeyData {
        layer_id: layer_id.clone(),
        neuron_id: neuron_id.clone(),
        round,
    };
    DataKey::NeuronResultKey(data)
}

pub fn get_submission_votes_key(submission_id: &String, round: u32) -> DataKey {
    let data = SubmissionVotesKeyData {
        submission_id: submission_id.clone(),
        round,
    };
    DataKey::SubmissionVotes(data)
}

pub fn get_submissions_key(round: u32) -> DataKey {
    let data = SubmissionsKeyData { round };
    DataKey::Submissions(data)
}

pub fn get_voting_powers_key(round: u32) -> DataKey {
    let data = VotingPowersKeyData { round };
    DataKey::VotingPowers(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn constructing_leyer_key() {
        let env = Env::default();

        let layer_id = String::from_str(&env, "1");

        let key = get_layer_key(&layer_id);
        assert_eq!(key, DataKey::LayerKey(LayerKeyData { layer_id }));
    }

    #[test]
    fn constructing_neuron_key() {
        let env = Env::default();

        let layer_id = String::from_str(&env, "1");
        let neuron_id = String::from_str(&env, "2");

        let key = get_neuron_key(&layer_id, &neuron_id);
        assert_eq!(
            key,
            DataKey::NeuronKey(NeuronKeyData {
                layer_id,
                neuron_id
            })
        );
    }

    #[test]
    fn constructing_neuron_result_key() {
        let env = Env::default();

        let layer_id = String::from_str(&env, "1");
        let neuron_id = String::from_str(&env, "2");
        let round = 25;

        let key = get_neuron_result_key(&layer_id, &neuron_id, round);
        assert_eq!(
            key,
            DataKey::NeuronResultKey(NeuronResultKeyData {
                layer_id,
                neuron_id,
                round
            })
        );
    }
}
