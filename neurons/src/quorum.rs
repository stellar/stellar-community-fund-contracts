use crate::Vote;
use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const SMALLEST_DEFINED_QUORUM_SIZE: usize = 7;
const MIN_QUORUM_SIZE: usize = 5;
const THRESHOLD: f64 = 0.667;

#[non_exhaustive]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct DelegateesForUser {
    delegatees: Vec<String>,
}

impl DelegateesForUser {
    #[must_use]
    pub fn new(delegatees: Vec<String>) -> Self {
        Self { delegatees }
    }
}

#[allow(clippy::implicit_hasher, clippy::missing_panics_doc)]
pub fn normalize_votes(
    votes: HashMap<String, HashMap<String, Vote>>,
    delegatees_for_user: &HashMap<String, DelegateesForUser>,
) -> Result<HashMap<String, HashMap<String, Vote>>> {
    votes
        .into_iter()
        .map(|(submission_name, submission_votes)| {
            let submission_votes =
                normalize_votes_for_submission(&submission_votes, delegatees_for_user)?;
            Ok((submission_name, submission_votes))
        })
        .collect::<Result<_>>()
}

fn normalize_votes_for_submission(
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
                let normalized_vote =
                    calculate_quorum_consensus(&user, &delegatees.delegatees, submission_votes)?;
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

    let mut selected_delegatees: Vec<&String> = delegatees
        .iter()
        .filter(|delegatee| {
            let delegatee_vote = submission_votes.get(*delegatee).unwrap_or(&Vote::Abstain);
            matches!(delegatee_vote, Vote::Yes | Vote::No)
        })
        .collect();

    while selected_delegatees.len() >= MIN_QUORUM_SIZE {
        let mut votes_yes = 0u32;
        let mut votes_no = 0u32;
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
        let total = f64::from(votes_yes + votes_no);
        if f64::from(votes_yes) / total > THRESHOLD {
            return Ok(Vote::Yes);
        }
        if f64::from(votes_no) / total > THRESHOLD {
            return Ok(Vote::No);
        }
        // Neither side reached the threshold — drop the last delegate and retry.
        selected_delegatees.pop();
    }

    Ok(Vote::Abstain)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_delegatees(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("del{i}")).collect()
    }

    fn votes_from(delegatees: &[String], votes: &[Vote]) -> HashMap<String, Vote> {
        assert_eq!(delegatees.len(), votes.len());
        delegatees
            .iter()
            .cloned()
            .zip(votes.iter().cloned())
            .collect()
    }

    #[test]
    fn resolves_yes_when_full_quorum_has_clear_yes_majority() {
        // 6 Yes / 1 No → 6/7 ≈ 0.857 > 0.667, no popping needed.
        let delegatees = make_delegatees(7);
        let submission_votes = votes_from(
            &delegatees,
            &[Vote::Yes, Vote::Yes, Vote::Yes, Vote::Yes, Vote::Yes, Vote::Yes, Vote::No],
        );

        let resolved =
            calculate_quorum_consensus("user", &delegatees, &submission_votes).unwrap();

        assert_eq!(resolved, Vote::Yes);
    }

    #[test]
    fn resolves_no_when_full_quorum_has_clear_no_majority() {
        // 1 Yes / 6 No → 6/7 ≈ 0.857 > 0.667, no popping needed.
        let delegatees = make_delegatees(7);
        let submission_votes = votes_from(
            &delegatees,
            &[Vote::Yes, Vote::No, Vote::No, Vote::No, Vote::No, Vote::No, Vote::No],
        );

        let resolved =
            calculate_quorum_consensus("user", &delegatees, &submission_votes).unwrap();

        assert_eq!(resolved, Vote::No);
    }

    #[test]
    fn resolves_yes_after_popping_trailing_dissenters() {
        // 4 Yes / 3 No, dissenters at the tail.
        // 7-wide:  4/7 ≈ 0.571  → pop No
        // 6-wide:  4/6 ≈ 0.667  → not strictly greater, pop No
        // 5-wide:  4/5 = 0.8    → Yes
        let delegatees = make_delegatees(7);
        let submission_votes = votes_from(
            &delegatees,
            &[Vote::Yes, Vote::Yes, Vote::Yes, Vote::Yes, Vote::No, Vote::No, Vote::No],
        );

        let resolved =
            calculate_quorum_consensus("user", &delegatees, &submission_votes).unwrap();

        assert_eq!(resolved, Vote::Yes);
    }

    #[test]
    fn resolves_no_after_popping_trailing_supporters() {
        // 3 No / 4 Yes, dissenters at the tail.
        // 7-wide:  4/7 ≈ 0.571 yes / 3/7 ≈ 0.429 no → pop Yes
        // 6-wide:  3/6 = 0.5   yes / 3/6 = 0.5   no → pop Yes
        // 5-wide:  2/5 = 0.4   yes / 3/5 = 0.6   no → still no consensus, pop Yes
        // 4-wide < MIN_QUORUM_SIZE → Abstain.
        // So we need a different layout to reach No: put No first.
        let delegatees = make_delegatees(7);
        let submission_votes = votes_from(
            &delegatees,
            &[Vote::No, Vote::No, Vote::No, Vote::No, Vote::Yes, Vote::Yes, Vote::Yes],
        );

        let resolved =
            calculate_quorum_consensus("user", &delegatees, &submission_votes).unwrap();

        assert_eq!(resolved, Vote::No);
    }

    #[test]
    fn resolves_abstain_when_no_pop_reaches_threshold() {
        // Alternating pattern keeps the split close at every iteration.
        // del1=Yes, del2=No, del3=Yes, del4=No, del5=Yes, del6=No, del7=Yes
        // 7-wide:  4Y/3N → 4/7 ≈ 0.571 → pop Yes (last)
        // 6-wide:  3Y/3N → 3/6 = 0.500 → pop No
        // 5-wide:  3Y/2N → 3/5 = 0.600 → pop Yes
        // 4-wide < MIN_QUORUM_SIZE → Abstain.
        let delegatees = make_delegatees(7);
        let submission_votes = votes_from(
            &delegatees,
            &[Vote::Yes, Vote::No, Vote::Yes, Vote::No, Vote::Yes, Vote::No, Vote::Yes],
        );

        let resolved =
            calculate_quorum_consensus("user", &delegatees, &submission_votes).unwrap();

        assert_eq!(resolved, Vote::Abstain);
    }

    #[test]
    fn resolves_abstain_when_too_few_delegates_actually_voted() {
        // Only 4 of 7 delegates have a Yes/No vote — the rest are absent (treated as Abstain
        // and filtered out). The filtered list starts below MIN_QUORUM_SIZE, so we never enter
        // the loop body and the result is Abstain.
        let delegatees = make_delegatees(7);
        let mut submission_votes = HashMap::new();
        submission_votes.insert(delegatees[0].clone(), Vote::Yes);
        submission_votes.insert(delegatees[1].clone(), Vote::Yes);
        submission_votes.insert(delegatees[2].clone(), Vote::Yes);
        submission_votes.insert(delegatees[3].clone(), Vote::Yes);

        let resolved =
            calculate_quorum_consensus("user", &delegatees, &submission_votes).unwrap();

        assert_eq!(resolved, Vote::Abstain);
    }

    #[test]
    fn errors_when_user_defines_fewer_than_smallest_quorum_size() {
        let delegatees = make_delegatees(SMALLEST_DEFINED_QUORUM_SIZE - 1);
        let submission_votes = HashMap::new();

        let result = calculate_quorum_consensus("user", &delegatees, &submission_votes);

        assert!(result.is_err());
    }

    #[test]
    fn non_voting_delegates_are_skipped_so_pop_chain_can_still_resolve() {
        // 9 delegates defined but only 7 cast Yes/No — the trailing 2 are filtered out,
        // leaving the same 4Y/3N tail as the "resolves yes after popping" test.
        let delegatees = make_delegatees(9);
        let mut submission_votes = HashMap::new();
        submission_votes.insert(delegatees[0].clone(), Vote::Yes);
        submission_votes.insert(delegatees[1].clone(), Vote::Yes);
        submission_votes.insert(delegatees[2].clone(), Vote::Yes);
        submission_votes.insert(delegatees[3].clone(), Vote::Yes);
        submission_votes.insert(delegatees[4].clone(), Vote::No);
        submission_votes.insert(delegatees[5].clone(), Vote::No);
        submission_votes.insert(delegatees[6].clone(), Vote::No);
        // del8, del9 — no recorded vote, should be skipped.

        let resolved =
            calculate_quorum_consensus("user", &delegatees, &submission_votes).unwrap();

        assert_eq!(resolved, Vote::Yes);
    }

    #[test]
    fn normalize_votes_for_submission_resolves_unanimous_yes_delegate() {
        let user0 = String::from("user0");
        let delegatees = make_delegatees(7);

        let mut submission_votes = HashMap::new();
        submission_votes.insert(user0.clone(), Vote::Delegate);
        for d in &delegatees {
            submission_votes.insert(d.clone(), Vote::Yes);
        }

        let mut delegates_for_user = HashMap::new();
        delegates_for_user.insert(user0.clone(), DelegateesForUser::new(delegatees));

        let normalized =
            normalize_votes_for_submission(&submission_votes, &delegates_for_user).unwrap();

        assert_eq!(normalized.get(&user0).unwrap(), &Vote::Yes);
    }

    #[test]
    fn normalize_votes_for_submission_resolves_unanimous_no_delegate() {
        let user0 = String::from("user0");
        let delegatees = make_delegatees(7);

        let mut submission_votes = HashMap::new();
        submission_votes.insert(user0.clone(), Vote::Delegate);
        for d in &delegatees {
            submission_votes.insert(d.clone(), Vote::No);
        }

        let mut delegates_for_user = HashMap::new();
        delegates_for_user.insert(user0.clone(), DelegateesForUser::new(delegatees));

        let normalized =
            normalize_votes_for_submission(&submission_votes, &delegates_for_user).unwrap();

        assert_eq!(normalized.get(&user0).unwrap(), &Vote::No);
    }

    #[test]
    fn normalize_votes_for_submission_resolves_abstain_when_split() {
        // Same alternating layout as `resolves_abstain_when_no_pop_reaches_threshold`,
        // but invoked through normalize_votes_for_submission.
        let user0 = String::from("user0");
        let delegatees = make_delegatees(7);

        let mut submission_votes = HashMap::new();
        submission_votes.insert(user0.clone(), Vote::Delegate);
        for (i, d) in delegatees.iter().enumerate() {
            let v = if i % 2 == 0 { Vote::Yes } else { Vote::No };
            submission_votes.insert(d.clone(), v);
        }

        let mut delegates_for_user = HashMap::new();
        delegates_for_user.insert(user0.clone(), DelegateesForUser::new(delegatees));

        let normalized =
            normalize_votes_for_submission(&submission_votes, &delegates_for_user).unwrap();

        assert_eq!(normalized.get(&user0).unwrap(), &Vote::Abstain);
    }
}
