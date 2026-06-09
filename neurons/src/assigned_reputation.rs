use crate::neurons::Neuron;
use serde_repr::Deserialize_repr;
use std::collections::HashMap;

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
    users_discord_roles: HashMap<String, Vec<String>>,
}

impl AssignedReputationNeuron {
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

fn reputation_bonus(reputation_tier: &ReputationTier) -> f64 {
    match reputation_tier {
        ReputationTier::Unknown | ReputationTier::Verified => 0.0,
        ReputationTier::Pathfinder => 1.0,
        ReputationTier::Navigator => 2.0,
        ReputationTier::Pilot => 3.0,
    }
}

fn discord_roles_bonus(roles: &Vec<String>) -> f64 {
    roles.iter().fold(0.0, |acc, role| acc + role_to_bonus(role))
}

fn role_to_bonus(role: &str) -> f64 {
    match role {
        "Ambassador President" => 1.0,
        "SCF Project" => 1.0,
        "Public Good Contributor" => 1.0,
        "Moderator" => 1.0,
        "SDF" => 1.0,
        "Tier 1 Validator" => 1.0,

        "West Africa Ambassador" => 0.5,
        "Brazil Ambassador" => 0.5,
        "India Ambassador" => 0.5,
        "Southern African Ambassador" => 0.5,
        "East Africa Ambassador" => 0.5,
        "Mexico Ambassador" => 0.5,
        "Colombia Ambassador" => 0.5,
        "Chile Ambassador" => 0.5,
        "Argentina Ambassador" => 0.5,
        "Europe Ambassador" => 0.5,

        _ => 0.0,
    }
}
impl Neuron for AssignedReputationNeuron {
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
    fn reputation_bonus_values() {
        assert_eq!(reputation_bonus(&ReputationTier::Unknown), 0.0);
        assert_eq!(reputation_bonus(&ReputationTier::Verified), 0.0);
        assert_eq!(reputation_bonus(&ReputationTier::Pathfinder), 1.0);
        assert_eq!(reputation_bonus(&ReputationTier::Navigator), 2.0);
        assert_eq!(reputation_bonus(&ReputationTier::Pilot), 3.0);
    }

    #[test]
    fn roles_bonus_values() {
        assert_eq!(discord_roles_bonus(&vec!["Ambassador President".to_string()]), 1.0);
        assert_eq!(discord_roles_bonus(&vec!["SCF Project".to_string()]), 1.0);
        assert_eq!(discord_roles_bonus(&vec!["Public Good Contributor".to_string()]), 1.0);
        assert_eq!(discord_roles_bonus(&vec!["Moderator".to_string()]), 1.0);
        assert_eq!(discord_roles_bonus(&vec!["SDF".to_string()]), 1.0);
        assert_eq!(discord_roles_bonus(&vec!["Tier 1 Validator".to_string()]), 1.0);

        assert_eq!(discord_roles_bonus(&vec!["West Africa Ambassador".to_string()]), 0.5);
        assert_eq!(discord_roles_bonus(&vec!["Brazil Ambassador".to_string()]), 0.5);
        assert_eq!(discord_roles_bonus(&vec!["India Ambassador".to_string()]), 0.5);
        assert_eq!(discord_roles_bonus(&vec!["Southern African Ambassador".to_string()]), 0.5);
        assert_eq!(discord_roles_bonus(&vec!["East Africa Ambassador".to_string()]), 0.5);
        assert_eq!(discord_roles_bonus(&vec!["Mexico Ambassador".to_string()]), 0.5);
        assert_eq!(discord_roles_bonus(&vec!["Colombia Ambassador".to_string()]), 0.5);
        assert_eq!(discord_roles_bonus(&vec!["Chile Ambassador".to_string()]), 0.5);
        assert_eq!(discord_roles_bonus(&vec!["Argentina Ambassador".to_string()]), 0.5);
        assert_eq!(discord_roles_bonus(&vec!["Europe Ambassador".to_string()]), 0.5);
    }

    #[test]
    fn neuron_run() {
        let users: Vec<String> = vec![
            "user1".to_string(),
            "user2".to_string(),
            "user3".to_string(),
        ];
        let mut users_reputation: HashMap<String, ReputationTier> = HashMap::new();
        users_reputation.insert("user1".to_string(), ReputationTier::Navigator);
        users_reputation.insert("user2".to_string(), ReputationTier::Pilot);
        users_reputation.insert("user3".to_string(), ReputationTier::Verified);

        let mut users_discord_roles: HashMap<String, Vec<String>> = HashMap::new();
        users_discord_roles.insert(
            "user1".to_string(),
            vec![
                "SDF".to_string(),
                "SCF Project".to_string(),
                "Moderator".to_string(),
            ],
        );
        users_discord_roles.insert("user2".to_string(), vec![]);
        users_discord_roles
            .insert("user3".to_string(), vec!["Public Good Contributor".to_string()]);

        let neuron = AssignedReputationNeuron::from_data(users_reputation, users_discord_roles);
        let resut = neuron.calculate_result(&users);

        assert_eq!(resut.get("user1").unwrap(), &5.0);
        assert_eq!(resut.get("user2").unwrap(), &3.0);
        assert_eq!(resut.get("user3").unwrap(), &1.0);
    }

    #[test]
    #[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
    fn missing_reputation_user_panics() {
        // NOTE: calculate_result does `.get(user).unwrap()` on users_reputation; a user missing
        // from that map panics. Documenting current behavior, not fixing.
        let neuron = AssignedReputationNeuron::from_data(HashMap::new(), HashMap::new());
        let _ = neuron.calculate_result(&["ghost".to_string()]);
    }

    #[test]
    #[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
    fn missing_discord_roles_user_panics() {
        // NOTE: `.get(user).unwrap()` on users_discord_roles also panics for a missing user.
        let mut reputation = HashMap::new();
        reputation.insert("alice".to_string(), ReputationTier::Pilot);
        let neuron = AssignedReputationNeuron::from_data(reputation, HashMap::new());
        let _ = neuron.calculate_result(&["alice".to_string()]);
    }

    #[test]
    fn multiple_roles_sum_with_reputation() {
        // confirms discord_roles_bonus SUMS across the whole vec (not just the first role)
        // and that the reputation bonus is added on top.
        let mut reputation = HashMap::new();
        reputation.insert("alice".to_string(), ReputationTier::Navigator); // 2.0
        let mut roles = HashMap::new();
        roles.insert(
            "alice".to_string(),
            vec![
                "SDF".to_string(),                    // 1.0
                "West Africa Ambassador".to_string(), // 0.5
                "Europe Ambassador".to_string(),      // 0.5
                "unrecognized role".to_string(),      // 0.0
            ],
        );
        let neuron = AssignedReputationNeuron::from_data(reputation, roles);
        let result = neuron.calculate_result(&["alice".to_string()]);
        // 2.0 + (1.0 + 0.5 + 0.5 + 0.0) == 4.0; exact f64 sum, so an epsilon compare is safe.
        assert!((result.get("alice").unwrap() - 4.0).abs() < 1e-9);
    }
}
