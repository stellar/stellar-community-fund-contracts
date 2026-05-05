#[cfg(test)]
use soroban_governor::GovernorContract;
use soroban_sdk::{testutils::Address as _, Address, Env};
use tests::{
    env::EnvTestUtils,
    governor::{create_governor, default_governor_settings},
};

#[test]
fn test_initialize_sets_storage() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let settings = default_governor_settings();
    let fixture = create_governor(&e, &bombadil, &bombadil, &settings);

    let result = fixture.governor.settings();
    assert_eq!(result.counting_type, settings.counting_type);
    assert_eq!(result.proposal_threshold, settings.proposal_threshold);
    assert_eq!(result.quorum, settings.quorum);
    assert_eq!(result.timelock, settings.timelock);
    assert_eq!(result.vote_delay, settings.vote_delay);
    assert_eq!(result.vote_period, settings.vote_period);
    assert_eq!(result.vote_threshold, settings.vote_threshold);

    assert_eq!(fixture.governor.council(), bombadil);
    assert_eq!(fixture.governor.vote_token(), fixture.token_address);
}

#[test]
#[should_panic(expected = "Error(Contract, #200)")]
fn test_initialize_vote_period_exceeds_max() {
    let e = Env::default();
    e.mock_all_auths();
    let votes = Address::generate(&e);
    let council = Address::generate(&e);
    let mut settings = default_governor_settings();
    settings.vote_period = 7 * 17280 + 1;

    e.register(GovernorContract, (votes, council, settings));
}

#[test]
#[should_panic(expected = "Error(Contract, #200)")]
fn test_initialize_proposal_exceeds_max_lifetime() {
    let e = Env::default();
    e.mock_all_auths();
    let votes = Address::generate(&e);
    let council = Address::generate(&e);
    let mut settings = default_governor_settings();
    settings.vote_delay = 5 * 17280;
    settings.vote_period = 5 * 17280;
    settings.timelock = 7 * 17280;
    settings.grace_period = 7 * 17280 + 1;

    e.register(GovernorContract, (votes, council, settings));
}
