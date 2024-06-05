use crate::Vote;
use anyhow::{anyhow, bail, Result};
use std::collections::HashMap;

const QUORUM_SIZE: u32 = 3;
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
