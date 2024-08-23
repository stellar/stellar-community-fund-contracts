use crate::Vote;
use anyhow::{anyhow, bail, Result};
use std::collections::HashMap;

const QUORUM_SIZE: u32 = 5;
const QUORUM_ABSOLUTE_PARTICIPATION_THRESHOLD: f64 = 1.0 / 2.0;
const QUORUM_RELATIVE_PARTICIPATION_THRESHOLD: f64 = 2.0 / 3.0;

#[allow(clippy::implicit_hasher)]
pub fn normalize_votes(
    votes: HashMap<String, HashMap<String, Vote>>,
    delegatees_for_user: &HashMap<String, Vec<String>>,
) -> Result<HashMap<String, HashMap<String, Vote>>> {
    votes
        .into_iter()
        .map(|(submission, submission_votes)| {
            let submission_votes =
                normalize_votes_for_submission(&submission_votes, delegatees_for_user)?;
            Ok((submission, submission_votes))
        })
        .collect::<Result<_>>()
}

fn normalize_votes_for_submission(
    submission_votes: &HashMap<String, Vote>,
    delegatees_for_user: &HashMap<String, Vec<String>>,
) -> Result<HashMap<String, Vote>> {
    submission_votes
        .clone()
        .into_iter()
        .map(|(user, vote)| {
            if vote == Vote::Delegate {
                let delegatees = delegatees_for_user
                    .get(&user)
                    .ok_or_else(|| anyhow!("Delegatees missing for user {user}"))?;
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
    } else {
        &valid_delegates[..QUORUM_SIZE as usize]
    };

    let mut quorum_size = 0;
    let mut agreement: i32 = 0;
    for &delegatee in selected_delegatees {
        let delegatee_vote = submission_votes.get(delegatee).unwrap_or(&Vote::Abstain);

        if delegatee_vote == &Vote::Delegate {
            continue;
        }

        quorum_size += 1;
        match delegatee_vote {
            Vote::Yes => agreement += 1,
            Vote::No => agreement -= 1,
            Vote::Abstain => {}
            Vote::Delegate => {
                bail!("Invalid delegatee operation");
            }
        };
    }

    let absolute_agreement: f64 = f64::from(agreement) / f64::from(QUORUM_SIZE);
    let relative_agreement: f64 = if quorum_size > 0 {
        f64::from(agreement) / f64::from(quorum_size)
    } else {
        0.0
    };

    Ok(
        if absolute_agreement.abs() > QUORUM_ABSOLUTE_PARTICIPATION_THRESHOLD {
            if relative_agreement.abs() > QUORUM_RELATIVE_PARTICIPATION_THRESHOLD {
                if relative_agreement > 0.0 {
                    Vote::Yes
                } else {
                    Vote::No
                }
            } else {
                Vote::Abstain
            }
        } else {
            Vote::Abstain
        },
    )
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
            vec![
                user2.clone(),
                user3.clone(),
                user4.clone(),
                user5.clone(),
                user6.clone(),
            ],
        );

        let normalized_votes =
            normalize_votes_for_submission(&submission_votes, &delegates_for_user).unwrap();

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
            vec![
                user2.clone(),
                user3.clone(),
                user4.clone(),
                user5.clone(),
                user6.clone(),
            ],
        );

        let normalized_votes =
            normalize_votes_for_submission(&submission_votes, &delegates_for_user).unwrap();

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
            vec![
                user2.clone(),
                user3.clone(),
                user4.clone(),
                user5.clone(),
                user6.clone(),
            ],
        );

        let normalized_votes =
            normalize_votes_for_submission(&submission_votes, &delegates_for_user).unwrap();

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
            vec![
                user2.clone(),
                user3.clone(),
                user4.clone(),
                user5.clone(),
                user6.clone(),
            ],
        );

        let normalized_votes =
            normalize_votes_for_submission(&submission_votes, &delegates_for_user).unwrap();

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

        delegates_for_user.insert(user1.clone(), vec![user2.clone(), user3.clone()]);

        let normalized_votes =
            normalize_votes_for_submission(&submission_votes, &delegates_for_user).unwrap();

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

        delegates_for_user.insert(user1.clone(), vec![user2.clone(), user3.clone()]);

        let normalized_votes =
            normalize_votes_for_submission(&submission_votes, &delegates_for_user).unwrap();

        assert_eq!(normalized_votes.get(&user1).unwrap(), &Vote::Abstain);
        assert_eq!(normalized_votes.get(&user2).unwrap(), &Vote::Yes);
        assert_eq!(normalized_votes.get(&user3).unwrap(), &Vote::Yes);
        assert_eq!(normalized_votes.get(&user4).unwrap(), &Vote::Yes);
        assert_eq!(normalized_votes.get(&user5).unwrap(), &Vote::Yes);
        assert_eq!(normalized_votes.get(&user6).unwrap(), &Vote::No);
    }
}
