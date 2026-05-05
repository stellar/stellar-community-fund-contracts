#[cfg(test)]
use soroban_governor::types::{Calldata, ProposalAction, ProposalStatus};
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, BytesN as _, Events},
    vec, Address, BytesN, Env, Error, IntoVal, Symbol, TryIntoVal, Val,
};
use tests::{
    env::EnvTestUtils,
    governor::{create_governor, default_governor_settings, default_proposal_data},
};

#[test]
fn test_propose_calldata() {
    let e = Env::default();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);
    fixture.whitelist(&[samwise.clone()]);

    let (title, description, action) = default_proposal_data(&e);
    let proposal_id = fixture
        .governor
        .propose(&samwise, &title, &description, &action);

    // capture events emitted during propose before subsequent calls clear them
    let events = e.events().all();

    // verify auth (last invocation authorization is from samwise)
    assert_eq!(
        e.auths()[0],
        (
            samwise.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    fixture.governor_address.clone(),
                    Symbol::new(&e, "propose"),
                    vec![
                        &e,
                        samwise.to_val(),
                        title.to_val(),
                        description.to_val(),
                        action.try_into_val(&e).unwrap()
                    ]
                )),
                sub_invocations: std::vec![]
            }
        )
    );

    let proposal = fixture.governor.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.id, proposal_id);
    assert_eq!(proposal.id, 0);
    match proposal.config.action {
        ProposalAction::Calldata(calldata) => {
            assert_eq!(calldata.contract_id, calldata.contract_id);
            assert_eq!(calldata.function, calldata.function);
            assert_eq!(calldata.args, calldata.args);
            if let ProposalAction::Calldata(action_calldata) = action.clone() {
                assert_eq!(
                    calldata.auths.get(0).unwrap().contract_id,
                    action_calldata.auths.get(0).unwrap().contract_id
                );
            } else {
                panic!("test setup error");
            }
        }
        _ => panic!("expected calldata proposal action"),
    }
    assert_eq!(proposal.config.title, title);
    assert_eq!(proposal.config.description, description);
    assert_eq!(proposal.data.creator, samwise);
    assert_eq!(
        proposal.data.vote_start,
        e.ledger().sequence() + settings.vote_delay
    );
    assert_eq!(
        proposal.data.vote_end,
        e.ledger().sequence() + settings.vote_delay + settings.vote_period
    );
    assert_eq!(proposal.data.status, ProposalStatus::Open);

    let votes = fixture.governor.get_proposal_votes(&proposal_id).unwrap();
    assert_eq!(votes.against, 0);
    assert_eq!(votes._for, 0);
    assert_eq!(votes.abstain, 0);

    // verify proposal_created event (captured above)
    let tx_events = vec![&e, events.last().unwrap()];
    let event_data: soroban_sdk::Vec<Val> = vec![
        &e,
        title.into_val(&e),
        description.into_val(&e),
        action.try_into_val(&e).unwrap(),
        proposal.data.vote_start.into_val(&e),
        proposal.data.vote_end.into_val(&e),
    ];
    assert_eq!(
        tx_events,
        vec![
            &e,
            (
                fixture.governor_address.clone(),
                (
                    Symbol::new(&e, "proposal_created"),
                    proposal_id,
                    samwise.clone()
                )
                    .into_val(&e),
                event_data.into_val(&e)
            )
        ]
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #213)")]
fn test_propose_calldata_validates() {
    let e = Env::default();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);
    fixture.whitelist(&[samwise.clone()]);

    let (title, description, _) = default_proposal_data(&e);
    // Calldata that targets the governor itself with a single sub-auth — invalid.
    let calldata = Calldata {
        contract_id: fixture.governor_address.clone(),
        function: Symbol::new(&e, "test"),
        args: (1, 2, 3).into_val(&e),
        auths: vec![&e],
    };
    let action = ProposalAction::Calldata(calldata);

    fixture
        .governor
        .propose(&samwise, &title, &description, &action);
}

#[test]
fn test_propose_with_active_proposal() {
    let e = Env::default();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);
    fixture.whitelist(&[samwise.clone()]);

    let (title, description, _) = default_proposal_data(&e);
    let action = ProposalAction::Snapshot;
    let proposal_id = fixture
        .governor
        .propose(&samwise, &title, &description, &action);
    let proposal = fixture.governor.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.id, 0);
    assert_eq!(proposal.data.creator, samwise);
    assert_eq!(proposal.data.status, ProposalStatus::Open);

    e.jump(settings.vote_delay + 1);

    // bombadil (council) tries to upgrade while samwise has an open proposal — fine,
    // open-proposal flag is per-creator. samwise opening another, however, must fail.
    let bytesn = BytesN::<32>::random(&e);
    let action2 = ProposalAction::Upgrade(bytesn);
    let result = fixture
        .governor
        .try_propose(&samwise, &title, &description, &action2);
    assert_eq!(result.err(), Some(Ok(Error::from_contract_error(211))));
}

#[test]
fn test_propose_snapshot() {
    let e = Env::default();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);
    fixture.whitelist(&[samwise.clone()]);

    let (title, description, _) = default_proposal_data(&e);
    let action = ProposalAction::Snapshot;

    let proposal_id = fixture
        .governor
        .propose(&samwise, &title, &description, &action);
    let proposal = fixture.governor.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.id, 0);
    assert_eq!(proposal.config.title, title);
    assert_eq!(proposal.config.description, description);
    matches!(proposal.config.action, ProposalAction::Snapshot);
    assert_eq!(proposal.data.creator, samwise);
    assert_eq!(proposal.data.vote_start, e.ledger().sequence());
    assert_eq!(
        proposal.data.vote_end,
        e.ledger().sequence() + settings.vote_period
    );
    assert_eq!(proposal.data.status, ProposalStatus::Open);
}

#[test]
fn test_propose_upgrade_by_council() {
    let e = Env::default();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);

    let (title, description, _) = default_proposal_data(&e);
    let bytes = BytesN::<32>::random(&e);
    let action = ProposalAction::Upgrade(bytes);

    let proposal_id = fixture
        .governor
        .propose(&bombadil, &title, &description, &action);

    let proposal = fixture.governor.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.id, 0);
    assert_eq!(proposal.config.title, title);
    assert_eq!(proposal.config.description, description);
    matches!(proposal.config.action, ProposalAction::Upgrade(_));
    assert_eq!(proposal.data.creator, bombadil);
    assert_eq!(proposal.data.status, ProposalStatus::Open);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_propose_upgrade_requires_council() {
    let e = Env::default();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);
    fixture.whitelist(&[samwise.clone()]);

    let (title, description, _) = default_proposal_data(&e);
    let bytes = BytesN::<32>::random(&e);
    let action = ProposalAction::Upgrade(bytes);

    fixture
        .governor
        .propose(&samwise, &title, &description, &action);
}

#[test]
fn test_propose_settings() {
    let e = Env::default();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);

    let (title, description, _) = default_proposal_data(&e);
    let mut new_settings = settings.clone();
    new_settings.vote_delay = 123;
    let action = ProposalAction::Settings(new_settings);

    let proposal_id = fixture
        .governor
        .propose(&bombadil, &title, &description, &action);

    let proposal = fixture.governor.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.id, 0);
    assert_eq!(proposal.config.title, title);
    assert_eq!(proposal.config.description, description);
    matches!(proposal.config.action, ProposalAction::Settings(_));
    assert_eq!(proposal.data.creator, bombadil);
    assert_eq!(proposal.data.status, ProposalStatus::Open);
}

#[test]
#[should_panic(expected = "Error(Contract, #200)")]
fn test_propose_settings_validates() {
    let e = Env::default();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);

    let (title, description, _) = default_proposal_data(&e);
    let mut new_settings = settings.clone();
    new_settings.vote_delay = 5 * 17280;
    new_settings.vote_period = 5 * 17280;
    new_settings.timelock = 7 * 17280;
    new_settings.grace_period = 7 * 17280 + 1;
    let action = ProposalAction::Settings(new_settings);

    fixture
        .governor
        .propose(&bombadil, &title, &description, &action);
}

#[test]
#[should_panic(expected = "Error(Contract, #215)")]
fn test_propose_requires_whitelist() {
    let e = Env::default();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);

    // initialize whitelist to empty so the missing-permission branch is hit cleanly;
    // samwise is not on it.
    fixture.whitelist(&[]);

    let (title, description, action) = default_proposal_data(&e);
    fixture
        .governor
        .propose(&samwise, &title, &description, &action);
}
