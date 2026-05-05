#[cfg(test)]
use soroban_governor::types::{Calldata, GovernorSettings, ProposalAction, ProposalStatus};
use soroban_sdk::{
    testutils::{Address as _, Events},
    vec, Address, Env, IntoVal, Symbol,
};
use tests::{
    env::EnvTestUtils,
    governor::{
        create_governor, create_mock_subcall_contract, create_sep41_token,
        default_governor_settings, default_proposal_data,
    },
};

const TOTAL_VOTES: i128 = 10_000 * 10i128.pow(7);

/// Helper: setup a fixture where samwise + pippin have voted on a `Settings` proposal
/// proposed by the council (bombadil). Returns `(fixture, proposal_id, new_settings)`.
fn setup_settings_proposal_voted(
    e: &Env,
) -> (tests::governor::GovernorFixture<'_>, u32, GovernorSettings) {
    let bombadil = Address::generate(e);
    let samwise = Address::generate(e);
    let pippin = Address::generate(e);

    let settings = default_governor_settings();
    let fixture = tests::governor::create_governor(e, &bombadil, &bombadil, &settings);

    let samwise_votes = 8_000 * 10i128.pow(7);
    let pippin_votes = 1_000 * 10i128.pow(7);
    let frodo_votes = TOTAL_VOTES - samwise_votes - pippin_votes;
    let frodo = Address::generate(e);
    fixture.set_voter_balance(&frodo, frodo_votes);
    fixture.set_voter_balance(&samwise, samwise_votes);
    fixture.set_voter_balance(&pippin, pippin_votes);

    let (title, description, _) = default_proposal_data(e);
    let new_settings = GovernorSettings {
        proposal_threshold: 829_421,
        vote_delay: 1231,
        vote_period: 7456,
        timelock: 15678,
        grace_period: 35678,
        quorum: 300,
        counting_type: 1,
        vote_threshold: 2000,
    };
    let action = ProposalAction::Settings(new_settings.clone());

    let proposal_id = fixture
        .governor
        .propose(&bombadil, &title, &description, &action);
    e.jump(settings.vote_delay + 1);
    fixture.governor.vote(&samwise, &proposal_id, &1);
    fixture.governor.vote(&pippin, &proposal_id, &0);
    e.jump(settings.vote_period);

    (fixture, proposal_id, new_settings)
}

#[test]
fn test_execute_settings() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let (fixture, proposal_id, new_settings) = setup_settings_proposal_voted(&e);
    let initial_settings = fixture.governor.settings();

    fixture.governor.close(&proposal_id);
    e.jump(initial_settings.timelock);

    fixture.governor.execute(&proposal_id);

    // capture events from execute before subsequent calls clear them
    let events = e.events().all();

    let gov_settings = fixture.governor.settings();
    assert_eq!(
        gov_settings.proposal_threshold,
        new_settings.proposal_threshold
    );
    assert_eq!(gov_settings.vote_delay, new_settings.vote_delay);
    assert_eq!(gov_settings.vote_period, new_settings.vote_period);
    assert_eq!(gov_settings.timelock, new_settings.timelock);
    assert_eq!(gov_settings.grace_period, new_settings.grace_period);
    assert_eq!(gov_settings.quorum, new_settings.quorum);
    assert_eq!(gov_settings.counting_type, new_settings.counting_type);
    assert_eq!(gov_settings.vote_threshold, new_settings.vote_threshold);

    let proposal = fixture.governor.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Executed);

    let tx_events = vec![&e, events.last().unwrap()];
    assert_eq!(
        tx_events,
        vec![
            &e,
            (
                fixture.governor_address.clone(),
                (Symbol::new(&e, "proposal_executed"), proposal_id).into_val(&e),
                ().into_val(&e)
            )
        ]
    );
}

#[test]
fn test_execute_expired() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let (fixture, proposal_id, _) = setup_settings_proposal_voted(&e);
    let initial_settings = fixture.governor.settings();

    fixture.governor.close(&proposal_id);
    e.jump(initial_settings.timelock);
    e.jump(initial_settings.grace_period + 1);

    fixture.governor.execute(&proposal_id);

    // capture events from execute before subsequent calls clear them
    let events = e.events().all();

    // settings should NOT have changed: new_settings.vote_delay = 1231 vs default ONE_DAY_LEDGERS
    assert_eq!(
        fixture.governor.settings().vote_delay,
        initial_settings.vote_delay
    );

    let proposal = fixture.governor.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Expired);

    let tx_events = vec![&e, events.last().unwrap()];
    assert_eq!(
        tx_events,
        vec![
            &e,
            (
                fixture.governor_address.clone(),
                (Symbol::new(&e, "proposal_expired"), proposal_id).into_val(&e),
                ().into_val(&e)
            )
        ]
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #201)")]
fn test_execute_nonexistent_proposal() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);

    fixture.governor.execute(&0);
}

#[test]
#[should_panic(expected = "Error(Contract, #205)")]
fn test_execute_proposal_not_queued() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let (fixture, proposal_id, _) = setup_settings_proposal_voted(&e);
    let initial_settings = fixture.governor.settings();
    // skip close
    e.jump(initial_settings.timelock);

    fixture.governor.execute(&proposal_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #206)")]
fn test_execute_timelock_not_met() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let (fixture, proposal_id, _) = setup_settings_proposal_voted(&e);
    let initial_settings = fixture.governor.settings();

    fixture.governor.close(&proposal_id);
    e.jump(initial_settings.timelock - 1);

    fixture.governor.execute(&proposal_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #205)")]
fn test_execute_defeated_errors() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let pippin = Address::generate(&e);
    let frodo = Address::generate(&e);

    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);

    let samwise_votes = 8_000 * 10i128.pow(7);
    let pippin_votes = 1_000 * 10i128.pow(7);
    let frodo_votes = TOTAL_VOTES - samwise_votes - pippin_votes;
    fixture.set_voter_balance(&frodo, frodo_votes);
    fixture.set_voter_balance(&samwise, samwise_votes);
    fixture.set_voter_balance(&pippin, pippin_votes);

    let (title, description, _) = default_proposal_data(&e);
    let mut new_settings = settings.clone();
    new_settings.vote_delay = 1231;
    let action = ProposalAction::Settings(new_settings);

    let proposal_id = fixture
        .governor
        .propose(&bombadil, &title, &description, &action);
    e.jump(settings.vote_delay + 1);
    fixture.governor.vote(&samwise, &proposal_id, &0);
    fixture.governor.vote(&pippin, &proposal_id, &1);
    e.jump(settings.vote_period);
    fixture.governor.close(&proposal_id);

    let proposal = fixture.governor.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Defeated);

    e.jump(settings.timelock);

    fixture.governor.execute(&proposal_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #205)")]
fn test_execute_snapshot_errors() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);

    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);
    fixture.whitelist(&[samwise.clone()]);

    let samwise_votes = 8_000 * 10i128.pow(7);
    let frodo = Address::generate(&e);
    let frodo_votes = TOTAL_VOTES - samwise_votes;
    fixture.set_voter_balance(&frodo, frodo_votes);
    fixture.set_voter_balance(&samwise, samwise_votes);

    let (title, description, _) = default_proposal_data(&e);
    let action = ProposalAction::Snapshot;

    let proposal_id = fixture
        .governor
        .propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay + 1);
    fixture.governor.vote(&samwise, &proposal_id, &1);
    e.jump(settings.vote_period);
    fixture.governor.close(&proposal_id);
    e.jump(settings.timelock);

    fixture.governor.execute(&proposal_id);
}

// --- calldata-execute auth-chain tests --------------------------------------
//
// These exercise the recursive auth tree that the governor declares via
// `e.authorize_as_current_contract(...)` when executing a Calldata proposal.
// They use a SEP-41 mock token as the underlying asset because the executed
// calldata invokes `token.transfer(governor, ...)`, which requires real
// `from.require_auth()` semantics — scf_token rejects transfers by design.

fn setup_calldata_fixture(
    e: &Env,
) -> (
    tests::governor::GovernorFixture<'_>,
    Address,
    soroban_sdk::token::StellarAssetClient<'_>,
    soroban_sdk::token::TokenClient<'_>,
    Address,
) {
    let bombadil = Address::generate(e);
    let samwise = Address::generate(e);

    let settings = default_governor_settings();
    let fixture = create_governor(e, &bombadil, &bombadil, &settings);
    fixture.whitelist(&[samwise.clone()]);

    let samwise_votes = 8_000 * 10i128.pow(7);
    let frodo_votes = TOTAL_VOTES - samwise_votes;
    let frodo = Address::generate(e);
    fixture.set_voter_balance(&frodo, frodo_votes);
    fixture.set_voter_balance(&samwise, samwise_votes);

    let (token_address, admin_client, token_client) = create_sep41_token(e, &bombadil);
    (fixture, samwise, admin_client, token_client, token_address)
}

#[test]
fn test_execute_calldata_no_auths() {
    let e = Env::default();
    e.set_default_info();

    let (fixture, samwise, admin_client, token_client, token_address) = setup_calldata_fixture(&e);
    let settings = fixture.governor.settings();

    let governor_transfer_amount: i128 = 10i128.pow(7);
    admin_client
        .mock_all_auths()
        .mint(&fixture.governor_address, &governor_transfer_amount);

    let (title, description, _) = default_proposal_data(&e);
    let action = ProposalAction::Calldata(Calldata {
        contract_id: token_address.clone(),
        function: Symbol::new(&e, "transfer"),
        args: (
            fixture.governor_address.clone(),
            samwise.clone(),
            governor_transfer_amount,
        )
            .into_val(&e),
        auths: vec![&e],
    });

    let proposal_id =
        fixture
            .governor
            .mock_all_auths()
            .propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay + 1);
    fixture
        .governor
        .mock_all_auths()
        .vote(&samwise, &proposal_id, &1);
    e.jump(settings.vote_period);
    fixture.governor.mock_all_auths().close(&proposal_id);
    e.jump(settings.timelock);

    // execute with no auth mocking — proves the governor's declared auth tree is sufficient
    e.set_auths(&[]);
    fixture.governor.set_auths(&[]);
    fixture.governor.execute(&proposal_id);

    let events = e.events().all();

    assert_eq!(token_client.balance(&samwise), governor_transfer_amount);
    assert_eq!(token_client.balance(&fixture.governor_address), 0);
    let proposal = fixture.governor.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Executed);

    let tx_events = vec![&e, events.last().unwrap()];
    assert_eq!(
        tx_events,
        vec![
            &e,
            (
                fixture.governor_address.clone(),
                (Symbol::new(&e, "proposal_executed"), proposal_id).into_val(&e),
                ().into_val(&e)
            )
        ]
    );
}

#[test]
fn test_execute_calldata_single_auth() {
    let e = Env::default();
    e.set_default_info();
    e.cost_estimate().budget().reset_unlimited();

    let (fixture, samwise, admin_client, token_client, token_address) = setup_calldata_fixture(&e);
    let settings = fixture.governor.settings();

    let (outer_subcall_address, _) =
        create_mock_subcall_contract(&e, &token_address, &fixture.governor_address);
    let (inner_subcall_address, _) =
        create_mock_subcall_contract(&e, &token_address, &fixture.governor_address);

    let call_amount: i128 = 100 * 10i128.pow(7);
    admin_client
        .mock_all_auths()
        .mint(&fixture.governor_address, &call_amount);

    // outer.call_subcall(inner, amount, auth=false) → inner.no_auth_sc(amount) → token.transfer(governor → inner)
    // The inner.no_auth_sc does NOT call require_auth on governor, so only the leaf transfer
    // needs governor auth — declared as a single flat sub-invocation.
    let (title, description, _) = default_proposal_data(&e);
    let action = ProposalAction::Calldata(Calldata {
        contract_id: outer_subcall_address.clone(),
        function: Symbol::new(&e, "call_subcall"),
        args: (inner_subcall_address.clone(), call_amount, false).into_val(&e),
        auths: vec![
            &e,
            Calldata {
                contract_id: token_address.clone(),
                function: Symbol::new(&e, "transfer"),
                args: (
                    fixture.governor_address.clone(),
                    inner_subcall_address.clone(),
                    call_amount,
                )
                    .into_val(&e),
                auths: vec![&e],
            },
        ],
    });

    let proposal_id =
        fixture
            .governor
            .mock_all_auths()
            .propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay + 1);
    fixture
        .governor
        .mock_all_auths()
        .vote(&samwise, &proposal_id, &1);
    e.jump(settings.vote_period);
    fixture.governor.mock_all_auths().close(&proposal_id);
    e.jump(settings.timelock);

    e.set_auths(&[]);
    fixture.governor.set_auths(&[]);
    fixture.governor.execute(&proposal_id);

    assert_eq!(token_client.balance(&inner_subcall_address), call_amount);
    assert_eq!(token_client.balance(&fixture.governor_address), 0);
    let proposal = fixture.governor.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Executed);
}

#[test]
fn test_execute_calldata_auth_chain() {
    let e = Env::default();
    e.set_default_info();
    e.cost_estimate().budget().reset_unlimited();

    let (fixture, samwise, admin_client, token_client, token_address) = setup_calldata_fixture(&e);
    let settings = fixture.governor.settings();

    let (outer_subcall_address, _) =
        create_mock_subcall_contract(&e, &token_address, &fixture.governor_address);
    let (inner_subcall_address, _) =
        create_mock_subcall_contract(&e, &token_address, &fixture.governor_address);

    let call_amount: i128 = 100 * 10i128.pow(7);
    admin_client
        .mock_all_auths()
        .mint(&fixture.governor_address, &call_amount);

    // outer.call_subcall(inner, amount, auth=true) → inner.subcall(amount) → token.transfer
    // Both inner.subcall AND token.transfer require governor auth, exercising the recursive
    // sub_invocations field of InvokerContractAuthEntry.
    let (title, description, _) = default_proposal_data(&e);
    let action = ProposalAction::Calldata(Calldata {
        contract_id: outer_subcall_address.clone(),
        function: Symbol::new(&e, "call_subcall"),
        args: (inner_subcall_address.clone(), call_amount, true).into_val(&e),
        auths: vec![
            &e,
            Calldata {
                contract_id: inner_subcall_address.clone(),
                function: Symbol::new(&e, "subcall"),
                args: (call_amount,).into_val(&e),
                auths: vec![
                    &e,
                    Calldata {
                        contract_id: token_address.clone(),
                        function: Symbol::new(&e, "transfer"),
                        args: (
                            fixture.governor_address.clone(),
                            inner_subcall_address.clone(),
                            call_amount,
                        )
                            .into_val(&e),
                        auths: vec![&e],
                    },
                ],
            },
        ],
    });

    let proposal_id =
        fixture
            .governor
            .mock_all_auths()
            .propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay + 1);
    fixture
        .governor
        .mock_all_auths()
        .vote(&samwise, &proposal_id, &1);
    e.jump(settings.vote_period);
    fixture.governor.mock_all_auths().close(&proposal_id);
    e.jump(settings.timelock);

    fixture.governor.set_auths(&[]);
    fixture.governor.execute(&proposal_id);

    assert_eq!(token_client.balance(&inner_subcall_address), call_amount);
    assert_eq!(token_client.balance(&fixture.governor_address), 0);
    let proposal = fixture.governor.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Executed);
}
