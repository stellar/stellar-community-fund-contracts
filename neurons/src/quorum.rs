use crate::{types::SubmissionCategory, Submission, Vote};
use anyhow::{anyhow, bail, Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::JsValue;
use web_sys::{
    self,
    console::{self, log_1},
};

const SMALLEST_DEFINED_QUORUM_SIZE: usize = 7;
const MIN_QUORUM_SIZE: usize = 5;
const THRESHOLD: f64 = 0.5;

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
                let normalized_vote =
                    calculate_quorum_consensus(&user, delegatees, submission_votes)?;
                Ok((user, normalized_vote))
            } else {
                Ok((user, vote))
            }
        })
        .collect::<Result<_>>()
}

fn calculate_quorum_consensus(
    user: &str,
    delegatees: &[String],
    submission_votes: &HashMap<String, Vote>,
) -> Result<Vote> {
    if delegatees.len() < SMALLEST_DEFINED_QUORUM_SIZE {
        bail!("User {} has quorum smaller than required {}", user, SMALLEST_DEFINED_QUORUM_SIZE)
    }

    let valid_delegates: Vec<&String> = delegatees
        .iter()
        .filter(|delegatee| {
            let delegatee_vote = submission_votes.get(*delegatee).unwrap_or(&Vote::Abstain);
            matches!(delegatee_vote, Vote::Yes | Vote::No)
        })
        .collect();

    // use the full qourum user has defined
    let selected_delegatees = valid_delegates;
    let mut resolved_vote = Vote::Abstain;

    while resolved_vote == Vote::Abstain {
        if selected_delegatees.len() < MIN_QUORUM_SIZE {
            break;
        }
        let mut votes_yes = 0;
        let mut votes_no = 0;
        for &delegatee in &selected_delegatees {
            let delegatee_vote = submission_votes.get(delegatee).unwrap_or(&Vote::Abstain);
            match delegatee_vote {
                Vote::Yes => votes_yes += 1,
                Vote::No => votes_no += 1,
                Vote::Abstain | Vote::Delegate => {
                    bail!("Invalid delegatee operation");
                }
            };
        }
        if votes_yes as f64 / (votes_yes + votes_no) as f64 > THRESHOLD {
            resolved_vote = Vote::Yes
        } else {
            resolved_vote = Vote::No
        }

        // With this calculation method Abstain will never occur. (assuming all delegates have voted)
        // But if we were to use some different method where Abstain could occur,
        // here we would pop one delegatee of selected_delegatees list, and
        // repeat untill we get Yes/No or run out of delegatees (min 5)

        // let _ = selected_delegatees.pop();
    }

    Ok(resolved_vote)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn expected_vote(submission_votes: &HashMap<String, Vote>) -> Vote {
        let mut yes = 0;
        let mut no = 0;
        for vote in submission_votes.values() {
            match vote {
                Vote::Yes => yes += 1,
                Vote::No => no += 1,
                _ => {}
            }
        }
        if yes as f64 / (yes + no) as f64 > THRESHOLD {
            Vote::Yes
        } else {
            Vote::No
        }
    }

    #[test]
    fn calculate_quorum_consensus_yes() {
        let delegatees: Vec<String> = vec![
            "del1".to_string(),
            "del2".to_string(),
            "del3".to_string(),
            "del4".to_string(),
            "del5".to_string(),
            "del6".to_string(),
            "del7".to_string(),
        ];
        let mut submission_votes: HashMap<String, Vote> = HashMap::new();
        submission_votes.insert("del1".to_string(), Vote::Yes);
        submission_votes.insert("del2".to_string(), Vote::Yes);
        submission_votes.insert("del3".to_string(), Vote::Yes);
        submission_votes.insert("del4".to_string(), Vote::Yes);
        submission_votes.insert("del5".to_string(), Vote::No);
        submission_votes.insert("del6".to_string(), Vote::No);
        submission_votes.insert("del7".to_string(), Vote::No);

        let resolved_vote =
            calculate_quorum_consensus("user", &delegatees, &submission_votes).unwrap();
        let expected_vote = expected_vote(&submission_votes);

        assert_eq!(resolved_vote, expected_vote)
    }

    #[test]
    fn calculate_quorum_consensus_no() {
        let delegatees: Vec<String> = vec![
            "del1".to_string(),
            "del2".to_string(),
            "del3".to_string(),
            "del4".to_string(),
            "del5".to_string(),
            "del6".to_string(),
            "del7".to_string(),
        ];
        let mut submission_votes: HashMap<String, Vote> = HashMap::new();
        submission_votes.insert("del1".to_string(), Vote::Yes);
        submission_votes.insert("del2".to_string(), Vote::Yes);
        submission_votes.insert("del3".to_string(), Vote::Yes);
        submission_votes.insert("del4".to_string(), Vote::No);
        submission_votes.insert("del5".to_string(), Vote::No);
        submission_votes.insert("del6".to_string(), Vote::No);
        submission_votes.insert("del7".to_string(), Vote::No);

        let resolved_vote =
            calculate_quorum_consensus("user", &delegatees, &submission_votes).unwrap();
        let expected_vote = expected_vote(&submission_votes);
        assert_eq!(resolved_vote, expected_vote)
    }

    #[test]
    fn abstain_if_less_than_x_delegates_voted() {
        let mut submission_votes = HashMap::new();

        let user0 = String::from("user0");
        let user1 = String::from("user1");
        let user2 = String::from("user2");
        let user3 = String::from("user3");
        let user4 = String::from("user4");
        let user5 = String::from("user5");
        let user6 = String::from("user6");
        let user7 = String::from("user7");

        submission_votes.insert(user0.clone(), Vote::Delegate);
        submission_votes.insert(user1.clone(), Vote::Yes);
        submission_votes.insert(user2.clone(), Vote::Yes);
        submission_votes.insert(user3.clone(), Vote::Yes);
        submission_votes.insert(user4.clone(), Vote::Yes);

        let delegates_for_user = vec![
            user1.clone(),
            user2.clone(),
            user3.clone(),
            user4.clone(),
            user5.clone(),
            user6.clone(),
            user7.clone(),
        ];
        let resolved_vote =
            calculate_quorum_consensus("user0", &delegates_for_user, &submission_votes).unwrap();

        assert_eq!(resolved_vote, Vote::Abstain);
    }

    #[test]
    fn quorum_size_to_small() {
        let mut submission_votes = HashMap::new();

        let user0 = String::from("user0");
        let user1 = String::from("user1");
        let user2 = String::from("user2");
        let user3 = String::from("user3");
        let user4 = String::from("user4");
        let user5 = String::from("user5");
        let user6 = String::from("user6");

        submission_votes.insert(user0.clone(), Vote::Delegate);

        let delegates_for_user = vec![
            user1.clone(),
            user2.clone(),
            user3.clone(),
            user4.clone(),
            user5.clone(),
            user6.clone(),
        ];

        let resolved_vote =
            calculate_quorum_consensus("user0", &delegates_for_user, &submission_votes);

        assert!(resolved_vote.is_err());
    }

    #[test]
    fn resolve_category_delegate_yes() {
        let mut submission_votes = HashMap::new();
        let mut delegates_for_user = HashMap::new();

        let user0 = String::from("user0");
        let user1 = String::from("user1");
        let user2 = String::from("user2");
        let user3 = String::from("user3");
        let user4 = String::from("user4");
        let user5 = String::from("user5");
        let user6 = String::from("user6");
        let user7 = String::from("user7");

        submission_votes.insert(user0.clone(), Vote::Delegate);
        submission_votes.insert(user1.clone(), Vote::Yes);
        submission_votes.insert(user2.clone(), Vote::Yes);
        submission_votes.insert(user3.clone(), Vote::Yes);
        submission_votes.insert(user4.clone(), Vote::Yes);
        submission_votes.insert(user5.clone(), Vote::Yes);
        submission_votes.insert(user6.clone(), Vote::Yes);
        submission_votes.insert(user7.clone(), Vote::Yes);

        delegates_for_user.insert(
            user0.clone(),
            DelegateesForUser::new(
                vec![
                    user1.clone(),
                    user2.clone(),
                    user3.clone(),
                    user4.clone(),
                    user5.clone(),
                    user6.clone(),
                    user7.clone(),
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

        let expected_vote = expected_vote(&submission_votes);
        assert_eq!(normalized_votes.get(&user0).unwrap(), &expected_vote);
    }

    #[test]
    fn resolve_category_delegate_no() {
        let mut submission_votes = HashMap::new();
        let mut delegates_for_user = HashMap::new();

        let user0 = String::from("user0");
        let user1 = String::from("user1");
        let user2 = String::from("user2");
        let user3 = String::from("user3");
        let user4 = String::from("user4");
        let user5 = String::from("user5");
        let user6 = String::from("user6");
        let user7 = String::from("user7");

        submission_votes.insert(user0.clone(), Vote::Delegate);
        submission_votes.insert(user1.clone(), Vote::No);
        submission_votes.insert(user2.clone(), Vote::No);
        submission_votes.insert(user3.clone(), Vote::No);
        submission_votes.insert(user4.clone(), Vote::No);
        submission_votes.insert(user5.clone(), Vote::No);
        submission_votes.insert(user6.clone(), Vote::No);
        submission_votes.insert(user7.clone(), Vote::No);

        delegates_for_user.insert(
            user0.clone(),
            DelegateesForUser::new(
                vec![
                    user1.clone(),
                    user2.clone(),
                    user3.clone(),
                    user4.clone(),
                    user5.clone(),
                    user6.clone(),
                    user7.clone(),
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

        let expected_vote = expected_vote(&submission_votes);
        assert_eq!(normalized_votes.get(&user0).unwrap(), &expected_vote);
    }

    #[test]
    fn non_voting_delegates_are_skipped_in_quorum() {
        let mut submission_votes = HashMap::new();
        let mut delegates_for_user = HashMap::new();

        let user0 = String::from("user0");
        let user1 = String::from("user1");
        let user2 = String::from("user2");
        let user3 = String::from("user3");
        let user4 = String::from("user4");
        let user5 = String::from("user5");
        let user6 = String::from("user6");
        let user7 = String::from("user7");
        let user8 = String::from("user8");
        let user9 = String::from("user9");

        submission_votes.insert(user0.clone(), Vote::Delegate);
        submission_votes.insert(user1.clone(), Vote::Yes);
        submission_votes.insert(user2.clone(), Vote::Yes);
        submission_votes.insert(user3.clone(), Vote::Yes);
        submission_votes.insert(user4.clone(), Vote::Yes);
        submission_votes.insert(user5.clone(), Vote::No);
        submission_votes.insert(user6.clone(), Vote::No);
        submission_votes.insert(user7.clone(), Vote::No);

        delegates_for_user.insert(
            user0.clone(),
            DelegateesForUser::new(
                vec![
                    user1.clone(),
                    user2.clone(),
                    user3.clone(),
                    user4.clone(),
                    user5.clone(),
                    user6.clone(),
                    user7.clone(),
                    user8.clone(),
                    user9.clone(),
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

        let expected_vote = expected_vote(&submission_votes);
        assert_eq!(normalized_votes.get(&user0).unwrap(), &expected_vote);
    }

    #[test]
    fn resolve_delegates_from_multiple_categories() {
        let mut submission0_votes = HashMap::new();
        let mut submission1_votes = HashMap::new();
        let mut delegates_for_user = HashMap::new();

        let user0 = String::from("user0");
        let user01 = String::from("user01");
        let user02 = String::from("user02");
        let user03 = String::from("user03");
        let user04 = String::from("user04");
        let user05 = String::from("user05");
        let user06 = String::from("user06");
        let user07 = String::from("user07");

        submission0_votes.insert(user0.clone(), Vote::Delegate);
        submission0_votes.insert(user01.clone(), Vote::Yes);
        submission0_votes.insert(user02.clone(), Vote::Yes);
        submission0_votes.insert(user03.clone(), Vote::Yes);
        submission0_votes.insert(user04.clone(), Vote::Yes);
        submission0_votes.insert(user05.clone(), Vote::Yes);
        submission0_votes.insert(user06.clone(), Vote::Yes);
        submission0_votes.insert(user07.clone(), Vote::Yes);

        let user11 = String::from("user11");
        let user12 = String::from("user12");
        let user13 = String::from("user13");
        let user14 = String::from("user14");
        let user15 = String::from("user15");
        let user16 = String::from("user16");
        let user17 = String::from("user17");

        submission1_votes.insert(user0.clone(), Vote::Delegate);
        submission1_votes.insert(user11.clone(), Vote::No);
        submission1_votes.insert(user12.clone(), Vote::No);
        submission1_votes.insert(user13.clone(), Vote::No);
        submission1_votes.insert(user14.clone(), Vote::No);
        submission1_votes.insert(user15.clone(), Vote::No);
        submission1_votes.insert(user16.clone(), Vote::No);
        submission1_votes.insert(user17.clone(), Vote::No);

        delegates_for_user.insert(
            user0.clone(),
            DelegateesForUser::new(
                vec![
                    user01.clone(),
                    user02.clone(),
                    user03.clone(),
                    user04.clone(),
                    user05.clone(),
                    user06.clone(),
                    user07.clone(),
                ],
                vec![],
                vec![],
                vec![
                    user11.clone(),
                    user12.clone(),
                    user13.clone(),
                    user14.clone(),
                    user15.clone(),
                    user16.clone(),
                    user17.clone(),
                ],
            ),
        );

        let normalized_votes_submission0 = normalize_votes_for_submission(
            &Submission::new("sub1".to_string(), SubmissionCategory::Applications, "".to_string()),
            &submission0_votes,
            &delegates_for_user,
        )
        .unwrap();

        let normalized_votes_submission1 = normalize_votes_for_submission(
            &Submission::new(
                "sub1".to_string(),
                SubmissionCategory::DeveloperTooling,
                "".to_string(),
            ),
            &submission1_votes,
            &delegates_for_user,
        )
        .unwrap();

        let expected_vote0 = expected_vote(&submission0_votes);
        assert_eq!(normalized_votes_submission0.get(&user0).unwrap(), &expected_vote0);

        let expected_vote1 = expected_vote(&submission1_votes);
        assert_eq!(normalized_votes_submission1.get(&user0).unwrap(), &expected_vote1);
    }
}
