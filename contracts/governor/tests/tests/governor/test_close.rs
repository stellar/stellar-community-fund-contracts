#[cfg(test)]
use soroban_governor::{storage, types::ProposalAction, types::ProposalStatus};
use soroban_sdk::{
    testutils::{Address as _, Events},
    vec, Address, Env, Error, IntoVal, Symbol,
};
use tests::{
    env::EnvTestUtils,
    governor::{create_governor, default_governor_settings, default_proposal_data},
};

const TOTAL_VOTES: i128 = 10_000 * 10i128.pow(7);

#[test]
fn test_close_successful() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);
    let samwise = Address::generate(&e);
    let pippin = Address::generate(&e);

    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);
    fixture.whitelist(&[samwise.clone()]);

    let samwise_votes = 105 * 10i128.pow(7);
    let pippin_votes = 100 * 10i128.pow(7);
    let frodo_votes = TOTAL_VOTES - samwise_votes - pippin_votes;
    fixture.set_voter_balance(&frodo, frodo_votes);
    fixture.set_voter_balance(&samwise, samwise_votes);
    fixture.set_voter_balance(&pippin, pippin_votes);

    let (title, description, action) = default_proposal_data(&e);

    let proposal_id = fixture
        .governor
        .propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay + 1);
    fixture.governor.vote(&samwise, &proposal_id, &1);
    fixture.governor.vote(&pippin, &proposal_id, &0);
    e.jump(settings.vote_period);

    fixture.governor.close(&proposal_id);

    // capture events from close before subsequent calls clear them
    let events = e.events().all();

    assert_eq!(e.auths().len(), 0);

    let proposal = fixture.governor.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Successful);
    assert_eq!(proposal.data.eta, e.ledger().sequence() + settings.timelock);

    let proposal_votes = fixture.governor.get_proposal_votes(&proposal_id);
    let tx_events = vec![&e, events.last().unwrap()];
    assert_eq!(
        tx_events,
        vec![
            &e,
            (
                fixture.governor_address.clone(),
                (
                    Symbol::new(&e, "proposal_voting_closed"),
                    proposal_id,
                    ProposalStatus::Successful as u32,
                    e.ledger().sequence() + settings.timelock
                )
                    .into_val(&e),
                proposal_votes.into_val(&e)
            )
        ]
    );

    // creator can create another proposal
    let proposal_id_new = fixture
        .governor
        .propose(&samwise, &title, &description, &action);
    assert_eq!(proposal_id_new, proposal_id + 1);
}

#[test]
fn test_close_defeated_quorum_not_met() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);
    let samwise = Address::generate(&e);
    let pippin = Address::generate(&e);

    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);
    fixture.whitelist(&[samwise.clone()]);

    let samwise_votes = 99 * 10i128.pow(7); // quorum is 1% — for-counted votes alone fall under
    let pippin_votes = 10 * 10i128.pow(7);
    let frodo_votes = TOTAL_VOTES - samwise_votes - pippin_votes;
    fixture.set_voter_balance(&frodo, frodo_votes);
    fixture.set_voter_balance(&samwise, samwise_votes);
    fixture.set_voter_balance(&pippin, pippin_votes);

    let (title, description, action) = default_proposal_data(&e);
    let proposal_id = fixture
        .governor
        .propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay + 1);
    fixture.governor.vote(&samwise, &proposal_id, &1);
    fixture.governor.vote(&pippin, &proposal_id, &0);
    e.jump(settings.vote_period);

    fixture.governor.close(&proposal_id);

    let proposal = fixture.governor.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Defeated);
    assert_eq!(proposal.data.eta, 0);
}

#[test]
fn test_close_defeated_threshold_not_met() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);
    let samwise = Address::generate(&e);
    let pippin = Address::generate(&e);
    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);
    fixture.whitelist(&[samwise.clone()]);

    let samwise_votes = 200 * 10i128.pow(7);
    let pippin_votes = 200 * 10i128.pow(7);
    let frodo_votes = TOTAL_VOTES - samwise_votes - pippin_votes;
    fixture.set_voter_balance(&frodo, frodo_votes);
    fixture.set_voter_balance(&samwise, samwise_votes);
    fixture.set_voter_balance(&pippin, pippin_votes);

    let (title, description, action) = default_proposal_data(&e);
    let proposal_id = fixture
        .governor
        .propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay + 1);
    fixture.governor.vote(&samwise, &proposal_id, &1);
    fixture.governor.vote(&pippin, &proposal_id, &0);
    e.jump(settings.vote_period);

    fixture.governor.close(&proposal_id);

    let proposal = fixture.governor.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Defeated);
    assert_eq!(proposal.data.eta, 0);
}

#[test]
fn test_close_expired() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);
    let samwise = Address::generate(&e);
    let pippin = Address::generate(&e);

    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);
    fixture.whitelist(&[samwise.clone()]);

    let samwise_votes = 105 * 10i128.pow(7);
    let pippin_votes = 100 * 10i128.pow(7);
    let frodo_votes = TOTAL_VOTES - samwise_votes - pippin_votes;
    fixture.set_voter_balance(&frodo, frodo_votes);
    fixture.set_voter_balance(&samwise, samwise_votes);
    fixture.set_voter_balance(&pippin, pippin_votes);

    let (title, description, action) = default_proposal_data(&e);
    let proposal_id = fixture
        .governor
        .propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay + 1);
    fixture.governor.vote(&samwise, &proposal_id, &1);
    fixture.governor.vote(&pippin, &proposal_id, &0);
    e.jump(settings.vote_period);
    e.jump(settings.grace_period + 1);

    fixture.governor.close(&proposal_id);

    // capture events from close before subsequent calls clear them
    let events = e.events().all();

    let proposal = fixture.governor.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Expired);
    assert_eq!(proposal.data.eta, 0);

    let proposal_votes = fixture.governor.get_proposal_votes(&proposal_id);
    let tx_events = events.slice((events.len() - 2)..events.len());
    assert_eq!(
        tx_events,
        vec![
            &e,
            (
                fixture.governor_address.clone(),
                (Symbol::new(&e, "proposal_expired"), proposal_id).into_val(&e),
                ().into_val(&e)
            ),
            (
                fixture.governor_address.clone(),
                (
                    Symbol::new(&e, "proposal_voting_closed"),
                    proposal_id,
                    ProposalStatus::Expired as u32,
                    0_u32
                )
                    .into_val(&e),
                proposal_votes.into_val(&e)
            )
        ]
    );
}

#[test]
fn test_close_tracks_quorum_with_counting_type() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);
    let samwise = Address::generate(&e);
    let pippin = Address::generate(&e);
    let merry = Address::generate(&e);

    let mut settings = default_governor_settings();
    settings.counting_type = 0b011; // include against and abstain in quorum
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);
    fixture.whitelist(&[samwise.clone()]);

    let samwise_votes = 50 * 10i128.pow(7);
    let pippin_votes = 10 * 10i128.pow(7);
    let merry_votes = 90 * 10i128.pow(7);
    let frodo_votes = TOTAL_VOTES - samwise_votes - pippin_votes - merry_votes;
    fixture.set_voter_balance(&frodo, frodo_votes);
    fixture.set_voter_balance(&samwise, samwise_votes);
    fixture.set_voter_balance(&pippin, pippin_votes);
    fixture.set_voter_balance(&merry, merry_votes);

    let (title, description, action) = default_proposal_data(&e);
    let proposal_id = fixture
        .governor
        .propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay + 1);
    fixture.governor.vote(&samwise, &proposal_id, &1);
    fixture.governor.vote(&pippin, &proposal_id, &0);
    fixture.governor.vote(&merry, &proposal_id, &2);
    e.jump(settings.vote_period);

    fixture.governor.close(&proposal_id);

    let proposal = fixture.governor.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Successful);
    assert_eq!(proposal.data.eta, e.ledger().sequence() + settings.timelock);
}

#[test]
fn test_close_successful_non_executable() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);
    let samwise = Address::generate(&e);
    let pippin = Address::generate(&e);

    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);
    fixture.whitelist(&[samwise.clone()]);

    let samwise_votes = 105 * 10i128.pow(7);
    let pippin_votes = 100 * 10i128.pow(7);
    let frodo_votes = TOTAL_VOTES - samwise_votes - pippin_votes;
    fixture.set_voter_balance(&frodo, frodo_votes);
    fixture.set_voter_balance(&samwise, samwise_votes);
    fixture.set_voter_balance(&pippin, pippin_votes);

    let (title, description, _) = default_proposal_data(&e);
    let action = ProposalAction::Snapshot;
    let proposal_id = fixture
        .governor
        .propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay + 1);
    fixture.governor.vote(&samwise, &proposal_id, &1);
    fixture.governor.vote(&pippin, &proposal_id, &0);
    e.jump(settings.vote_period);

    fixture.governor.close(&proposal_id);

    let proposal = fixture.governor.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Successful);
    assert_eq!(proposal.data.eta, 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #201)")]
fn test_close_nonexistent_proposal() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);

    let proposal_id = e.as_contract(&fixture.governor_address, || {
        storage::get_next_proposal_id(&e)
    });
    fixture.governor.close(&proposal_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #204)")]
fn test_close_vote_period_unfinished() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

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
    fixture.governor.vote(&samwise, &proposal_id, &1);
    e.jump(settings.vote_period - 1);

    fixture.governor.close(&proposal_id);
}

#[test]
fn test_close_already_closed() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);
    let samwise = Address::generate(&e);
    let pippin = Address::generate(&e);

    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);
    fixture.whitelist(&[samwise.clone()]);

    let samwise_votes = 105 * 10i128.pow(7);
    let pippin_votes = 100 * 10i128.pow(7);
    let frodo_votes = TOTAL_VOTES - samwise_votes - pippin_votes;
    fixture.set_voter_balance(&frodo, frodo_votes);
    fixture.set_voter_balance(&samwise, samwise_votes);
    fixture.set_voter_balance(&pippin, pippin_votes);

    let (title, description, action) = default_proposal_data(&e);
    let proposal_id = fixture
        .governor
        .propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay + 1);
    fixture.governor.vote(&samwise, &proposal_id, &1);
    fixture.governor.vote(&pippin, &proposal_id, &0);
    e.jump(settings.vote_period);

    fixture.governor.close(&proposal_id);

    let proposal = fixture.governor.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Successful);

    let result = fixture.governor.try_close(&proposal_id);
    assert_eq!(result.err(), Some(Ok(Error::from_contract_error(202))));
}
