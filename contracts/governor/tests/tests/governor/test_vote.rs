#[cfg(test)]
use soroban_governor::types::ProposalStatus;
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events},
    vec, Address, Env, IntoVal, Symbol, TryIntoVal, Val,
};
use tests::{
    env::EnvTestUtils,
    governor::{create_governor, default_governor_settings, default_proposal_data},
};

#[test]
fn test_vote() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);

    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);
    fixture.whitelist(&[samwise.clone()]);

    let samwise_votes = 8_000 * 10i128.pow(7);
    fixture.set_voter_balance(&samwise, samwise_votes);

    let (title, description, action) = default_proposal_data(&e);
    let proposal_id = fixture
        .governor
        .propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay + 1);

    let voter_support = 0;
    fixture
        .governor
        .vote(&samwise, &proposal_id, &voter_support);

    // capture events from vote before subsequent calls clear them
    let events = e.events().all();

    assert_eq!(
        e.auths()[0],
        (
            samwise.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    fixture.governor_address.clone(),
                    Symbol::new(&e, "vote"),
                    vec![
                        &e,
                        samwise.to_val(),
                        proposal_id.try_into_val(&e).unwrap(),
                        voter_support.try_into_val(&e).unwrap()
                    ]
                )),
                sub_invocations: std::vec![]
            }
        )
    );

    let votes = fixture.governor.get_vote(&samwise, &proposal_id);
    assert_eq!(votes, Some(voter_support));
    let proposal = fixture.governor.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Open);
    let vote_count = fixture.governor.get_proposal_votes(&proposal_id).unwrap();
    assert_eq!(vote_count.against, samwise_votes);
    assert_eq!(vote_count._for, 0);
    assert_eq!(vote_count.abstain, 0);

    let tx_events = vec![&e, events.last().unwrap()];
    let event_data: soroban_sdk::Vec<Val> =
        vec![&e, voter_support.into_val(&e), samwise_votes.into_val(&e)];
    assert_eq!(
        tx_events,
        vec![
            &e,
            (
                fixture.governor_address.clone(),
                (Symbol::new(&e, "vote_cast"), proposal_id, samwise.clone()).into_val(&e),
                event_data.into_val(&e)
            )
        ]
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #209)")]
fn test_vote_user_changes_support() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);
    fixture.whitelist(&[samwise.clone()]);

    fixture.set_voter_balance(&samwise, 8_000 * 10i128.pow(7));

    let (title, description, action) = default_proposal_data(&e);
    let proposal_id = fixture
        .governor
        .propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay + 1);

    fixture.governor.vote(&samwise, &proposal_id, &0);
    fixture.governor.vote(&samwise, &proposal_id, &1);
}

#[test]
fn test_vote_multiple_users() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let pippin = Address::generate(&e);
    let merry = Address::generate(&e);
    let bilbo = Address::generate(&e);

    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);
    fixture.whitelist(&[samwise.clone()]);

    let samwise_votes = 1_000 * 10i128.pow(7);
    let pippin_votes = 500 * 10i128.pow(7);
    let merry_votes = 1_234_567;
    let bilbo_votes = 2345 * 10i128.pow(7);

    fixture.set_voter_balance(&samwise, samwise_votes);
    fixture.set_voter_balance(&pippin, pippin_votes);
    fixture.set_voter_balance(&merry, merry_votes);
    fixture.set_voter_balance(&bilbo, bilbo_votes);

    let (title, description, action) = default_proposal_data(&e);
    let proposal_id = fixture
        .governor
        .propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay + 1);

    fixture.governor.vote(&samwise, &proposal_id, &1);
    e.jump(10);
    fixture.governor.vote(&pippin, &proposal_id, &0);
    e.jump(123);
    fixture.governor.vote(&merry, &proposal_id, &0);
    fixture.governor.vote(&bilbo, &proposal_id, &2);
    e.jump(50);

    assert_eq!(fixture.governor.get_vote(&samwise, &proposal_id), Some(1));
    assert_eq!(fixture.governor.get_vote(&pippin, &proposal_id), Some(0));
    assert_eq!(fixture.governor.get_vote(&merry, &proposal_id), Some(0));
    assert_eq!(fixture.governor.get_vote(&bilbo, &proposal_id), Some(2));
    let proposal = fixture.governor.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Open);
    let vote_count = fixture.governor.get_proposal_votes(&proposal_id).unwrap();
    assert_eq!(vote_count.against, pippin_votes + merry_votes);
    assert_eq!(vote_count._for, samwise_votes);
    assert_eq!(vote_count.abstain, bilbo_votes);
}

#[test]
#[should_panic(expected = "Error(Contract, #201)")]
fn test_vote_nonexistent_proposal() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);

    fixture.governor.vote(&samwise, &0, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #212)")]
fn test_vote_delay_not_ended() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);
    fixture.whitelist(&[samwise.clone()]);

    fixture.set_voter_balance(&samwise, 8_000 * 10i128.pow(7));

    let (title, description, action) = default_proposal_data(&e);
    let proposal_id = fixture
        .governor
        .propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay - 1);

    fixture.governor.vote(&samwise, &proposal_id, &1);
}

#[test]
#[should_panic(expected = "Error(Contract, #212)")]
fn test_vote_period_ended() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);
    fixture.whitelist(&[samwise.clone()]);

    fixture.set_voter_balance(&samwise, 8_000 * 10i128.pow(7));

    let (title, description, action) = default_proposal_data(&e);
    let proposal_id = fixture
        .governor
        .propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay);
    e.jump(settings.vote_period + 1);

    fixture.governor.vote(&samwise, &proposal_id, &1);
}

#[test]
#[should_panic(expected = "Error(Contract, #203)")]
fn test_vote_invalid_support_option() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);

    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);
    fixture.whitelist(&[samwise.clone()]);

    fixture.set_voter_balance(&samwise, 8_000 * 10i128.pow(7));

    let (title, description, action) = default_proposal_data(&e);
    let proposal_id = fixture
        .governor
        .propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay + 1);

    fixture.governor.vote(&samwise, &proposal_id, &3);
}

#[test]
#[should_panic(expected = "Error(Contract, #208)")]
fn test_vote_zero_balance() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let pippin = Address::generate(&e);
    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);
    fixture.whitelist(&[samwise.clone()]);

    // Only samwise has a balance — pippin tries to vote with zero, must fail.
    fixture.set_voter_balance(&samwise, 8_000 * 10i128.pow(7));

    let (title, description, action) = default_proposal_data(&e);
    let proposal_id = fixture
        .governor
        .propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay + 1);

    fixture.governor.vote(&pippin, &proposal_id, &1);
}
