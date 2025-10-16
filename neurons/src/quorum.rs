use crate::{types::SubmissionCategory, Submission, Vote};
use anyhow::{anyhow, bail, Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::JsValue;
use web_sys::{
    self,
    console::{self, log_1},
};

const QUORUM_SIZE: u32 = 5;
const QUORUM_ABSOLUTE_PARTICIPATION_THRESHOLD: f64 = 1.0 / 2.0;
const QUORUM_RELATIVE_PARTICIPATION_THRESHOLD: f64 = 2.0 / 3.0;

#[non_exhaustive]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct DelegateesForUser {
    applications: Vec<String>,
    financial_protocols: Vec<String>,
    infrastructure_and_services: Vec<String>,
    developer_tooling: Vec<String>,
}

impl DelegateesForUser {
    #[must_use]
    pub fn new(
        applications: Vec<String>,
        financial_protocols: Vec<String>,
        infrastructure_and_services: Vec<String>,
        developer_tooling: Vec<String>,
    ) -> Self {
        Self {
            applications,
            financial_protocols,
            infrastructure_and_services,
            developer_tooling,
        }
    }
}

#[allow(clippy::implicit_hasher, clippy::missing_panics_doc)]
pub fn normalize_votes(
    votes: HashMap<String, HashMap<String, Vote>>,
    submissions: &[Submission],
    delegatees_for_user: &HashMap<String, DelegateesForUser>,
) -> Result<HashMap<String, HashMap<String, Vote>>> {
    votes
        .into_iter()
        .map(|(submission_name, submission_votes)| {
            let submission = submissions
                .iter()
                .find(|sub| sub.name == submission_name)
                .expect(&format!("Missing details for submission: {}", submission_name));
            let submission_votes =
                normalize_votes_for_submission(submission, &submission_votes, delegatees_for_user)?;
            Ok((submission_name, submission_votes))
        })
        .collect::<Result<_>>()
}

fn delegatees_for_category<'a>(
    submission_category: &SubmissionCategory,
    delegatees_for_user: &'a DelegateesForUser,
) -> &'a Vec<String> {
    match submission_category {
        SubmissionCategory::Applications => &delegatees_for_user.applications,
        SubmissionCategory::FinancialProtocols => &delegatees_for_user.financial_protocols,
        SubmissionCategory::InfrastructureAndServices => {
            &delegatees_for_user.infrastructure_and_services
        }
        SubmissionCategory::DeveloperTooling => &delegatees_for_user.developer_tooling,
    }
}

fn normalize_votes_for_submission(
    submission: &Submission,
    submission_votes: &HashMap<String, Vote>,
    delegatees_for_user: &HashMap<String, DelegateesForUser>,
) -> Result<HashMap<String, Vote>> {
    submission_votes
        .clone()
        .into_iter()
        .map(|(user, vote)| {
            if vote == Vote::Delegate {
                let delegatees = delegatees_for_user
                    .get(&user)
                    .ok_or_else(|| anyhow!("Delegatees missing for user {user}"))?;
                let delegatees = delegatees_for_category(&submission.category, delegatees);
                let normalized_vote = calculate_quorum_consensus(delegatees, submission_votes)?;
                Ok((user, normalized_vote))
            } else {
                Ok((user, vote))
            }
        })
        .collect::<Result<_>>()
}

fn calculate_quorum_consensus(
    delegatees: &[String],
    submission_votes: &HashMap<String, Vote>,
) -> Result<Vote> {
    let valid_delegates: Vec<&String> = delegatees
        .iter()
        .filter(|delegatee| {
            let delegatee_vote = submission_votes.get(*delegatee).unwrap_or(&Vote::Abstain);
            matches!(delegatee_vote, Vote::Yes | Vote::No)
        })
        .collect();

    let selected_delegatees = if valid_delegates.len() < QUORUM_SIZE as usize {
        valid_delegates.as_slice()
        // bail!("not enough delegates defined");
    } else {
        &valid_delegates[..QUORUM_SIZE as usize]
    };

    // log_1(&JsValue::from_str(&format!(
    //     "valid_delegates {} selected_delegatees {}",
    //     valid_delegates.len(),
    //     selected_delegatees.len()
    // )));

    let mut votes_yes = 0;
    let mut votes_no = 0;
    for &delegatee in selected_delegatees {
        let delegatee_vote = submission_votes.get(delegatee).unwrap_or(&Vote::Abstain);

        match delegatee_vote {
            Vote::Yes => votes_yes += 1,
            Vote::No => votes_no += 1,
            Vote::Abstain | Vote::Delegate => {
                bail!("Invalid delegatee operation");
            }
        };
    }

    if votes_yes > votes_no {
        Ok(Vote::Yes)
    } else if votes_yes < votes_no {
        Ok(Vote::No)
    } else {
        Ok(Vote::Abstain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_delegate_yes() {
        let mut submission_votes = HashMap::new();
        let mut delegates_for_user = HashMap::new();

        let user1 = String::from("user1");
        let user2 = String::from("user2");
        let user3 = String::from("user3");
        let user4 = String::from("user4");
        let user5 = String::from("user5");
        let user6 = String::from("user6");
        // let user7 = String::from("user7");
        // let user8 = String::from("user8");
        // let user9 = String::from("user9");
        // let user10 = String::from("user10");

        submission_votes.insert(user1.clone(), Vote::Delegate);
        submission_votes.insert(user2.clone(), Vote::Yes);
        submission_votes.insert(user3.clone(), Vote::Yes);
        submission_votes.insert(user4.clone(), Vote::Yes);
        submission_votes.insert(user5.clone(), Vote::Yes);
        submission_votes.insert(user6.clone(), Vote::Yes);

        delegates_for_user.insert(
            user1.clone(),
            DelegateesForUser::new(
                vec![
                    user2.clone(),
                    user3.clone(),
                    user4.clone(),
                    user5.clone(),
                    user6.clone(),
                ],
                vec![],
                vec![],
                vec![],
            ),
        );

        let normalized_votes = normalize_votes_for_submission(
            &Submission::new("sub1".to_string(), SubmissionCategory::Applications, "".to_string()),
            &submission_votes,
            &delegates_for_user,
        )
        .unwrap();

        assert_eq!(normalized_votes.get(&user1).unwrap(), &Vote::Yes);
        assert_eq!(normalized_votes.get(&user2).unwrap(), &Vote::Yes);
        assert_eq!(normalized_votes.get(&user3).unwrap(), &Vote::Yes);
        assert_eq!(normalized_votes.get(&user4).unwrap(), &Vote::Yes);
        assert_eq!(normalized_votes.get(&user5).unwrap(), &Vote::Yes);
        assert_eq!(normalized_votes.get(&user6).unwrap(), &Vote::Yes);
    }

    #[test]
    fn resolve_delegate_no() {
        let mut submission_votes = HashMap::new();
        let mut delegates_for_user = HashMap::new();

        let user1 = String::from("user1");
        let user2 = String::from("user2");
        let user3 = String::from("user3");
        let user4 = String::from("user4");
        let user5 = String::from("user5");
        let user6 = String::from("user6");

        submission_votes.insert(user1.clone(), Vote::Delegate);
        submission_votes.insert(user2.clone(), Vote::No);
        submission_votes.insert(user3.clone(), Vote::No);
        submission_votes.insert(user4.clone(), Vote::No);
        submission_votes.insert(user5.clone(), Vote::No);
        submission_votes.insert(user6.clone(), Vote::No);

        delegates_for_user.insert(
            user1.clone(),
            DelegateesForUser::new(
                vec![
                    user2.clone(),
                    user3.clone(),
                    user4.clone(),
                    user5.clone(),
                    user6.clone(),
                ],
                vec![],
                vec![],
                vec![],
            ),
        );

        let normalized_votes = normalize_votes_for_submission(
            &Submission::new("sub1".to_string(), SubmissionCategory::Applications, "".to_string()),
            &submission_votes,
            &delegates_for_user,
        )
        .unwrap();

        assert_eq!(normalized_votes.get(&user1).unwrap(), &Vote::No);
        assert_eq!(normalized_votes.get(&user2).unwrap(), &Vote::No);
        assert_eq!(normalized_votes.get(&user3).unwrap(), &Vote::No);
        assert_eq!(normalized_votes.get(&user4).unwrap(), &Vote::No);
        assert_eq!(normalized_votes.get(&user5).unwrap(), &Vote::No);
        assert_eq!(normalized_votes.get(&user6).unwrap(), &Vote::No);
    }

    #[test]
    fn resolve_delegate_abstain() {
        let mut submission_votes = HashMap::new();
        let mut delegates_for_user = HashMap::new();

        let user1 = String::from("user1");
        let user2 = String::from("user2");
        let user3 = String::from("user3");
        let user4 = String::from("user4");
        let user5 = String::from("user5");
        let user6 = String::from("user6");

        // Agreement = 3 - 2 = 1
        // Relative agreement = 1 / 5
        // Absolute agreement = 1 / 5
        submission_votes.insert(user1.clone(), Vote::Delegate);
        submission_votes.insert(user2.clone(), Vote::Yes);
        submission_votes.insert(user3.clone(), Vote::Yes);
        submission_votes.insert(user4.clone(), Vote::Yes);
        submission_votes.insert(user5.clone(), Vote::No);
        submission_votes.insert(user6.clone(), Vote::No);

        delegates_for_user.insert(
            user1.clone(),
            DelegateesForUser::new(
                vec![
                    user2.clone(),
                    user3.clone(),
                    user4.clone(),
                    user5.clone(),
                    user6.clone(),
                ],
                vec![],
                vec![],
                vec![],
            ),
        );

        let normalized_votes = normalize_votes_for_submission(
            &Submission::new("sub1".to_string(), SubmissionCategory::Applications, "".to_string()),
            &submission_votes,
            &delegates_for_user,
        )
        .unwrap();

        assert_eq!(normalized_votes.get(&user1).unwrap(), &Vote::Abstain);
        assert_eq!(normalized_votes.get(&user2).unwrap(), &Vote::Yes);
        assert_eq!(normalized_votes.get(&user3).unwrap(), &Vote::Yes);
        assert_eq!(normalized_votes.get(&user4).unwrap(), &Vote::Yes);
        assert_eq!(normalized_votes.get(&user5).unwrap(), &Vote::No);
        assert_eq!(normalized_votes.get(&user6).unwrap(), &Vote::No);
    }

    #[test]
    fn non_voting_delegates_are_ignored() {
        let mut submission_votes = HashMap::new();
        let mut delegates_for_user = HashMap::new();

        let user1 = String::from("user1");
        let user2 = String::from("user2");
        let user3 = String::from("user3");
        let user4 = String::from("user4");
        let user5 = String::from("user5");
        let user6 = String::from("user6");

        // Agreement = 3
        // Relative agreement = 3 / 3
        // Absolute agreement = 3 / 5
        submission_votes.insert(user1.clone(), Vote::Delegate);
        submission_votes.insert(user2.clone(), Vote::Abstain);
        submission_votes.insert(user3.clone(), Vote::Abstain);
        submission_votes.insert(user4.clone(), Vote::Yes);
        submission_votes.insert(user5.clone(), Vote::Yes);
        submission_votes.insert(user6.clone(), Vote::Yes);

        delegates_for_user.insert(
            user1.clone(),
            DelegateesForUser::new(
                vec![
                    user2.clone(),
                    user3.clone(),
                    user4.clone(),
                    user5.clone(),
                    user6.clone(),
                ],
                vec![],
                vec![],
                vec![],
            ),
        );

        let normalized_votes = normalize_votes_for_submission(
            &Submission::new("sub1".to_string(), SubmissionCategory::Applications, "".to_string()),
            &submission_votes,
            &delegates_for_user,
        )
        .unwrap();

        assert_eq!(normalized_votes.get(&user1).unwrap(), &Vote::Yes);
        assert_eq!(normalized_votes.get(&user2).unwrap(), &Vote::Abstain);
        assert_eq!(normalized_votes.get(&user3).unwrap(), &Vote::Abstain);
        assert_eq!(normalized_votes.get(&user4).unwrap(), &Vote::Yes);
        assert_eq!(normalized_votes.get(&user5).unwrap(), &Vote::Yes);
        assert_eq!(normalized_votes.get(&user6).unwrap(), &Vote::Yes);
    }

    #[test]
    fn relative_threshold_not_passed() {
        let mut submission_votes = HashMap::new();
        let mut delegates_for_user = HashMap::new();

        let user1 = String::from("user1");
        let user2 = String::from("user2");
        let user3 = String::from("user3");

        // Agreement = 2
        // Relative agreement = 2 / 2
        // Absolute agreement = 2 / 5
        submission_votes.insert(user1.clone(), Vote::Delegate);
        submission_votes.insert(user2.clone(), Vote::Yes);
        submission_votes.insert(user3.clone(), Vote::Yes);

        delegates_for_user.insert(
            user1.clone(),
            DelegateesForUser::new(vec![user2.clone(), user3.clone()], vec![], vec![], vec![]),
        );

        let normalized_votes = normalize_votes_for_submission(
            &Submission::new("sub1".to_string(), SubmissionCategory::Applications, "".to_string()),
            &submission_votes,
            &delegates_for_user,
        )
        .unwrap();

        assert_eq!(normalized_votes.get(&user1).unwrap(), &Vote::Abstain);
        assert_eq!(normalized_votes.get(&user2).unwrap(), &Vote::Yes);
        assert_eq!(normalized_votes.get(&user3).unwrap(), &Vote::Yes);
    }

    #[test]
    fn absolute_threshold_not_passed() {
        let mut submission_votes = HashMap::new();
        let mut delegates_for_user = HashMap::new();

        let user1 = String::from("user1");
        let user2 = String::from("user2");
        let user3 = String::from("user3");
        let user4 = String::from("user4");
        let user5 = String::from("user5");
        let user6 = String::from("user6");

        // Agreement = 4 - 1 = 3
        // Relative agreement = 3 / 5
        // Absolute agreement = 3 / 5
        submission_votes.insert(user1.clone(), Vote::Delegate);
        submission_votes.insert(user2.clone(), Vote::Yes);
        submission_votes.insert(user3.clone(), Vote::Yes);
        submission_votes.insert(user4.clone(), Vote::Yes);
        submission_votes.insert(user5.clone(), Vote::Yes);
        submission_votes.insert(user6.clone(), Vote::No);

        delegates_for_user.insert(
            user1.clone(),
            DelegateesForUser::new(vec![user2.clone(), user3.clone()], vec![], vec![], vec![]),
        );

        let normalized_votes = normalize_votes_for_submission(
            &Submission::new("sub1".to_string(), SubmissionCategory::Applications, "".to_string()),
            &submission_votes,
            &delegates_for_user,
        )
        .unwrap();

        assert_eq!(normalized_votes.get(&user1).unwrap(), &Vote::Abstain);
        assert_eq!(normalized_votes.get(&user2).unwrap(), &Vote::Yes);
        assert_eq!(normalized_votes.get(&user3).unwrap(), &Vote::Yes);
        assert_eq!(normalized_votes.get(&user4).unwrap(), &Vote::Yes);
        assert_eq!(normalized_votes.get(&user5).unwrap(), &Vote::Yes);
        assert_eq!(normalized_votes.get(&user6).unwrap(), &Vote::No);
    }

    #[test]
    fn resolve_delegates_from_multiple_categories() {
        let mut submission1_votes = HashMap::new();
        let mut submission2_votes = HashMap::new();
        let mut delegates_for_user = HashMap::new();

        let user1 = String::from("user1");
        let user2 = String::from("user2");
        let user3 = String::from("user3");
        let user4 = String::from("user4");
        let user5 = String::from("user5");
        let user6 = String::from("user6");
        let user7 = String::from("user7");
        let user8 = String::from("user8");
        let user9 = String::from("user9");
        let user10 = String::from("user10");

        submission1_votes.insert(user1.clone(), Vote::Delegate);
        submission1_votes.insert(user2.clone(), Vote::Yes);
        submission1_votes.insert(user3.clone(), Vote::Yes);
        submission1_votes.insert(user4.clone(), Vote::Yes);
        submission1_votes.insert(user5.clone(), Vote::Yes);
        submission1_votes.insert(user6.clone(), Vote::Yes);

        submission2_votes.insert(user1.clone(), Vote::Delegate);
        submission2_votes.insert(user7.clone(), Vote::No);
        submission2_votes.insert(user8.clone(), Vote::No);
        submission2_votes.insert(user9.clone(), Vote::No);
        submission2_votes.insert(user10.clone(), Vote::No);

        delegates_for_user.insert(
            user1.clone(),
            DelegateesForUser::new(
                vec![
                    user2.clone(),
                    user3.clone(),
                    user4.clone(),
                    user5.clone(),
                    user6.clone(),
                ],
                vec![user7.clone(), user8.clone(), user9.clone(), user10.clone()],
                vec![],
                vec![],
            ),
        );

        let normalized_votes_for_submission1 = normalize_votes_for_submission(
            &Submission::new("sub1".to_string(), SubmissionCategory::Applications, "".to_string()),
            &submission1_votes,
            &delegates_for_user,
        )
        .unwrap();
        let normalized_votes_for_submission2 = normalize_votes_for_submission(
            &Submission::new(
                "sub2".to_string(),
                SubmissionCategory::FinancialProtocols,
                "".to_string(),
            ),
            &submission2_votes,
            &delegates_for_user,
        )
        .unwrap();

        assert_eq!(normalized_votes_for_submission1.get(&user1).unwrap(), &Vote::Yes);
        assert_eq!(normalized_votes_for_submission1.get(&user2).unwrap(), &Vote::Yes);
        assert_eq!(normalized_votes_for_submission1.get(&user3).unwrap(), &Vote::Yes);
        assert_eq!(normalized_votes_for_submission1.get(&user4).unwrap(), &Vote::Yes);
        assert_eq!(normalized_votes_for_submission1.get(&user5).unwrap(), &Vote::Yes);
        assert_eq!(normalized_votes_for_submission1.get(&user6).unwrap(), &Vote::Yes);

        assert_eq!(normalized_votes_for_submission2.get(&user1).unwrap(), &Vote::No);
        assert_eq!(normalized_votes_for_submission2.get(&user7).unwrap(), &Vote::No);
        assert_eq!(normalized_votes_for_submission2.get(&user8).unwrap(), &Vote::No);
        assert_eq!(normalized_votes_for_submission2.get(&user9).unwrap(), &Vote::No);
        assert_eq!(normalized_votes_for_submission2.get(&user10).unwrap(), &Vote::No);
    }
}
