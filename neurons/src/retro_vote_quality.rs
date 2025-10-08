use crate::neurons::Neuron;
use crate::Vote;
use std::collections::HashMap;

const DELEGATED_VOTE_DENOMINATOR: i32 = 2;
const FIXED_POINT_SCALING_FACTOR: i32 = 100; // *10 to mitigate float precission loss, and *10 to allow integer division
#[derive(Clone, Debug)]
pub struct RetroVoteQualityNeuron {
    votes_per_round: HashMap<u32, HashMap<String, HashMap<String, Vote>>>, // round -> submission -> user -> vote (Yes/No/Abstain/Delegate)
    normalized_votes_per_round: HashMap<u32, HashMap<String, HashMap<String, Vote>>>, // round -> submission -> user -> vote (Yes/No/Abstain)
    tranche_status_map: HashMap<String, Vec<String>>,
    submissions_airtable_ids: HashMap<String, String>,
}

impl RetroVoteQualityNeuron {
    pub fn from_data(
        votes_per_round: HashMap<u32, HashMap<String, HashMap<String, Vote>>>,
        normalized_votes_per_round: HashMap<u32, HashMap<String, HashMap<String, Vote>>>,
        tranche_status_map: HashMap<String, Vec<String>>,
        submissions_airtable_ids: HashMap<String, String>,
    ) -> Self {
        Self {
            votes_per_round,
            normalized_votes_per_round,
            tranche_status_map,
            submissions_airtable_ids,
        }
    }
    fn run_user(&self, user: &str) -> f64 {
        let mut total_bonus: i32 = 0;
        // loop through rounds 30-current
        for (round, round_votes) in &self.votes_per_round {
            // loop through all submissions
            for (submission_name, submission_votes) in round_votes {
                // loop through all votes
                for (voter, vote) in submission_votes {
                    // skip votes from other users, and no/abstain
                    if voter != user || vote == &Vote::No || vote == &Vote::Abstain {
                        continue;
                    };
                    // lookup bonus for this submission
                    let bonus_value: i32 = match self.lookup_tranche_status(&submission_name) {
                        Some(tranche_status) => tranche_status_to_bonus(&tranche_status),
                        None => continue,
                    };
                    match vote {
                        // apply bonus value
                        Vote::Yes => total_bonus += bonus_value,
                        // or resolve delegation
                        Vote::Delegate => {
                            // lookup this round-submission-user vote in normalized_votes_per_round
                            if let Some(resolved_vote) =
                                self.resolve_delegated_vote(*round, &submission_name, user)
                            {
                                // apply bonus value * 0.5
                                if resolved_vote == Vote::Yes {
                                    total_bonus += bonus_value / DELEGATED_VOTE_DENOMINATOR;
                                }
                            }
                        }
                        Vote::Abstain | Vote::No => {}
                    }
                }
            }
        }
        total_bonus as f64 / FIXED_POINT_SCALING_FACTOR as f64
    }
    fn resolve_delegated_vote(
        &self,
        round: u32,
        submission_name: &str,
        user: &str,
    ) -> Option<Vote> {
        let round_votes = match self.normalized_votes_per_round.get(&round) {
            Some(round_votes) => round_votes,
            None => {
                return None;
            }
        };
        let submission_votes = match round_votes.get(submission_name) {
            Some(submission_votes) => submission_votes,
            None => return None,
        };
        match submission_votes.get(user) {
            Some(vote) => return Some(vote.clone()),
            None => return None,
        }
    }
    fn lookup_tranche_status(&self, submission_name: &str) -> Option<String> {
        // lookup airtable id of the submission
        if let Some(airtable_id) = self.submissions_airtable_ids.get(submission_name) {
            // lookup tranche status
            for (status, airtable_ids) in &self.tranche_status_map {
                if airtable_ids.contains(airtable_id) {
                    return Some(status.to_string());
                }
            }
        }
        None
    }
}
fn tranche_status_to_bonus(tranche_status: &str) -> i32 {
    match tranche_status {
        "Live on Stellar within 6 months" => 30,               // 0.3
        "Live on Stellar after 6 months" => 10,                // 0.1
        "Not live on Stellar within 6 months, Awarded" => -30, // -0.3
        "Not live on Stellar within 6 months, MVP" => -20,     // -0.2
        "Not live on Stellar within 6 months, Testnet" => -10, // -0.1
        _ => 0,
    }
}
impl Neuron for RetroVoteQualityNeuron {
    fn name(&self) -> String {
        "retro_vote_quality_neuron".to_string()
    }

    fn calculate_result(&self, users: &[String]) -> HashMap<String, f64> {
        let mut result = HashMap::new();

        for user in users {
            let bonus = self.run_user(user);
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
