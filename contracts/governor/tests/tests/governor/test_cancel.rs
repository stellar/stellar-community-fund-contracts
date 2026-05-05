#[cfg(test)]
use soroban_governor::types::ProposalStatus;
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events},
    vec, Address, Env, Error, IntoVal, Symbol, TryIntoVal,
};
use tests::{
    env::EnvTestUtils,
    governor::{create_governor, default_governor_settings, default_proposal_data},
};

#[test]
fn test_cancel() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);
    fixture.whitelist(&[samwise.clone()]);

    let (title, description, action) = default_proposal_data(&e);

    let proposal_id = fixture
        .governor
        .propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay / 2);

    fixture.governor.cancel(&samwise, &proposal_id);

    // capture events from cancel before subsequent calls clear them
    let events = e.events().all();

    assert_eq!(
        e.auths()[0],
        (
            samwise.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    fixture.governor_address.clone(),
                    Symbol::new(&e, "cancel"),
                    vec![&e, samwise.to_val(), proposal_id.try_into_val(&e).unwrap()]
                )),
                sub_invocations: std::vec![]
            }
        )
    );

    let proposal = fixture.governor.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Canceled);

    let tx_events = vec![&e, events.last().unwrap()];
    assert_eq!(
        tx_events,
        vec![
            &e,
            (
                fixture.governor_address.clone(),
                (Symbol::new(&e, "proposal_canceled"), proposal_id).into_val(&e),
                ().into_val(&e)
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
fn test_cancel_council() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);
    fixture.whitelist(&[samwise.clone()]);

    let (title, description, action) = default_proposal_data(&e);

    let proposal_id = fixture
        .governor
        .propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay / 2);

    fixture.governor.cancel(&bombadil, &proposal_id);

    let proposal = fixture.governor.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Canceled);

    let proposal_id_new = fixture
        .governor
        .propose(&samwise, &title, &description, &action);
    assert_eq!(proposal_id_new, proposal_id + 1);
}

#[test]
#[should_panic(expected = "Error(Contract, #201)")]
fn test_cancel_nonexistent_proposal() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);

    fixture.governor.cancel(&samwise, &1);
}

#[test]
#[should_panic(expected = "Error(Contract, #207)")]
fn test_cancel_proposal_active() {
    let e = Env::default();
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

    fixture.governor.vote(&samwise, &proposal_id, &1);

    fixture.governor.cancel(&samwise, &proposal_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_cancel_unauthorized_address() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let pippin = Address::generate(&e);
    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);
    fixture.whitelist(&[samwise.clone()]);

    let (title, description, action) = default_proposal_data(&e);

    let proposal_id = fixture
        .governor
        .propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay / 2);

    fixture.governor.cancel(&pippin, &proposal_id);
}

#[test]
fn test_cancel_already_closed() {
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

    let total_votes: i128 = 10_000 * 10i128.pow(7);
    let samwise_votes = 105 * 10i128.pow(7);
    let pippin_votes = 100 * 10i128.pow(7);
    let frodo_votes = total_votes - samwise_votes - pippin_votes;
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

    let result = fixture.governor.try_cancel(&samwise, &proposal_id);
    assert_eq!(result.err(), Some(Ok(Error::from_contract_error(202))));
}
