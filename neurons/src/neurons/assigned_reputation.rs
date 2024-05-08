use crate::neurons::Neuron;
use anyhow::Result;
use camino::Utf8Path;
use serde_repr::Deserialize_repr;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

#[derive(Deserialize_repr, Clone, Debug)]
#[repr(i32)]
pub enum ReputationTier {
    Unknown = -1,
    Verified = 0,
    Pathfinder = 1,
    Navigator = 2,
    Pilot = 3,
}

#[derive(Clone, Debug)]
pub struct AssignedReputationNeuron {
    users_reputation: HashMap<String, ReputationTier>,
}

impl AssignedReputationNeuron {
    pub fn try_from_file(path: &Utf8Path) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let users_reputation = serde_json::from_reader(reader)?;
        Ok(Self { users_reputation })
    }
}

fn reputation_bonus(reputation_tier: &ReputationTier) -> f64 {
    match reputation_tier {
        ReputationTier::Unknown | ReputationTier::Verified => 0.0,
        ReputationTier::Pathfinder => 0.1,
        ReputationTier::Navigator => 0.2,
        ReputationTier::Pilot => 0.3,
    }
}

impl Neuron for AssignedReputationNeuron {
    fn name(&self) -> String {
        "assigned_reputation_neuron".to_string()
    }

    fn calculate_result(&self, users: &[String]) -> HashMap<String, f64> {
        let mut result = HashMap::new();

        for user in users {
            let bonus = reputation_bonus(self.users_reputation.get(user).unwrap());
            result.insert(user.into(), bonus);
        }

        result
    }
}
