use crate::neurons::Neuron;
use serde_repr::Deserialize_repr;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct RetroVoteQualityNeuron {
    users_reputation: HashMap<String, ReputationTier>,
    users_discord_roles: HashMap<String, Vec<String>>,
}

impl RetroVoteQualityNeuron {
    pub fn from_data(
        users_reputation: HashMap<String, ReputationTier>,
        users_discord_roles: HashMap<String, Vec<String>>,
    ) -> Self {
        Self {
            users_reputation,
            users_discord_roles,
        }
    }
}

impl Neuron for RetroVoteQualityNeuron {
    fn name(&self) -> String {
        "assigned_reputation_neuron".to_string()
    }

    fn calculate_result(&self, users: &[String]) -> HashMap<String, f64> {
        let mut result = HashMap::new();

        for user in users {
            let mut bonus = reputation_bonus(self.users_reputation.get(user).unwrap());
            bonus += discord_roles_bonus(self.users_discord_roles.get(user).unwrap());
            result.insert(user.into(), bonus);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reputation_bonus_values() {}
}
