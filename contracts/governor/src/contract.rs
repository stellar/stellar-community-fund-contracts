use soroban_sdk::{
    contract, contractimpl, panic_with_error, unwrap::UnwrapOptimized, Address, Env, String,
};

use crate::{
    dependencies::VotesClient,
    errors::GovernorError,
    events::GovernorEvents,
    governor::Governor,
    settings::require_valid_settings,
    storage,
    types::{
        GovernorSettings, Proposal, ProposalAction, ProposalConfig, ProposalData, ProposalStatus,
        VoteCount,
    },
};

#[contract]
pub struct GovernorContract;

#[contractimpl]
impl Governor for GovernorContract {
    fn initialize(e: Env, votes: Address, council: Address, settings: GovernorSettings) {
        if storage::get_is_init(&e) {
            panic_with_error!(&e, GovernorError::AlreadyInitializedError);
        }
        require_valid_settings(&e, &settings);
        storage::set_settings(&e, &settings);
        storage::set_council_address(&e, &council);
        storage::set_voter_token_address(&e, &votes);
        storage::set_is_init(&e);
        storage::extend_instance(&e);
    }

    fn settings(e: Env) -> GovernorSettings {
        storage::get_settings(&e)
    }

    fn council(e: Env) -> Address {
        storage::get_council_address(&e)
    }

    fn vote_token(e: Env) -> Address {
        storage::get_voter_token_address(&e)
    }

    fn propose(
        e: Env,
        creator: Address,
        title: String,
        description: String,
        action: ProposalAction,
    ) -> u32 {
        creator.require_auth();
        storage::extend_instance(&e);

        if storage::has_open_proposal(&e, &creator) {
            panic_with_error!(&e, GovernorError::ProposalAlreadyOpenError);
        }

        match action {
            ProposalAction::Upgrade(_) | ProposalAction::Settings(_) => {
                let council = storage::get_council_address(&e);
                if creator != council {
                    panic_with_error!(&e, GovernorError::UnauthorizedError);
                }
            }
            ProposalAction::Council(_) => {
                // we don't allow anyone to propose a council change
                panic_with_error!(&e, GovernorError::ProposalActionNotSupported);
            }
            _ => {}
        };
        let settings = storage::get_settings(&e);
        let votes_client = VotesClient::new(&e, &storage::get_voter_token_address(&e));
        let creater_votes = votes_client.get_votes(&creator);
        if creater_votes < settings.proposal_threshold {
            panic_with_error!(&e, GovernorError::InsufficientVotingUnitsError)
        }

        let proposal_config =
            ProposalConfig::new(&e, title.clone(), description.clone(), action.clone());
        let proposal_id = storage::get_next_proposal_id(&e);
        let vote_start = match action {
            // no vote delay for snapshot proposals as they cannot be executed
            ProposalAction::Snapshot => e.ledger().sequence(),
            // all other proposals have a vote delay
            _ => e.ledger().sequence() + settings.vote_delay,
        };
        let vote_end = vote_start + settings.vote_period;
        let proposal_data = ProposalData {
            creator: creator.clone(),
            vote_start,
            vote_end,
            eta: 0,
            status: ProposalStatus::Open,
            executable: proposal_config.is_executable(),
        };
        storage::set_next_proposal_id(&e, proposal_id + 1);

        storage::create_proposal_config(&e, proposal_id, &proposal_config);
        storage::create_proposal_data(&e, proposal_id, &proposal_data);
        storage::create_proposal_vote_count(&e, proposal_id);
        storage::create_open_proposal(&e, &creator);

        votes_client.set_vote_sequence(&vote_start);

        GovernorEvents::proposal_created(
            &e,
            proposal_id,
            creator,
            title,
            description,
            action,
            vote_start,
            vote_end,
        );
        proposal_id
    }

    fn get_proposal(e: Env, proposal_id: u32) -> Option<Proposal> {
        let config = storage::get_proposal_config(&e, proposal_id);
        let data = storage::get_proposal_data(&e, proposal_id);
        if config.is_none() || data.is_none() {
            None
        } else {
            Some(Proposal {
                id: proposal_id,
                config: config.unwrap_optimized(),
                data: data.unwrap_optimized(),
            })
        }
    }

    fn close(e: Env, proposal_id: u32) {
        storage::extend_instance(&e);
        let mut proposal_data = storage::get_proposal_data(&e, proposal_id)
            .unwrap_or_else(|| panic_with_error!(&e, GovernorError::NonExistentProposalError));

        if proposal_data.status != ProposalStatus::Open {
            panic_with_error!(&e, GovernorError::ProposalClosedError);
        }

        if e.ledger().sequence() <= proposal_data.vote_end {
            panic_with_error!(&e, GovernorError::VotePeriodNotFinishedError)
        }

        let settings = storage::get_settings(&e);
        let vote_count = storage::get_proposal_vote_count(&e, proposal_id).unwrap_optimized();
        if e.ledger().sequence() > proposal_data.vote_end + settings.grace_period {
            // proposal took too long to be closed. Mark expired and close.
            proposal_data.status = ProposalStatus::Expired;
            GovernorEvents::proposal_expired(&e, proposal_id);
        } else {
            // proposal closed in time. Check if it passed or failed.
            let votes_client = VotesClient::new(&e, &storage::get_voter_token_address(&e));
            let total_vote_supply = votes_client.get_past_total_supply(&proposal_data.vote_start);

            let passed_quorum = vote_count.is_over_quorum(
                settings.quorum,
                settings.counting_type,
                total_vote_supply,
            );
            let passed_vote_threshold = vote_count.is_over_threshold(settings.vote_threshold);

            if passed_vote_threshold && passed_quorum {
                proposal_data.status = ProposalStatus::Successful;
                if proposal_data.executable {
                    proposal_data.eta = e.ledger().sequence() + settings.timelock;
                }
            } else {
                proposal_data.status = ProposalStatus::Defeated;
            }
        }

        storage::set_proposal_data(&e, proposal_id, &proposal_data);
        storage::del_open_proposal(&e, &proposal_data.creator);
        GovernorEvents::proposal_voting_closed(
            &e,
            proposal_id,
            proposal_data.status as u32,
            proposal_data.eta,
            vote_count,
        );
    }

    fn execute(e: Env, proposal_id: u32) {
        storage::extend_instance(&e);
        let mut proposal_data = storage::get_proposal_data(&e, proposal_id)
            .unwrap_or_else(|| panic_with_error!(&e, GovernorError::NonExistentProposalError));

        if proposal_data.status != ProposalStatus::Successful
            || !proposal_data.executable
            || proposal_data.eta == 0
        {
            panic_with_error!(&e, GovernorError::ProposalNotExecutableError);
        }

        if e.ledger().sequence() < proposal_data.eta {
            panic_with_error!(&e, GovernorError::TimelockNotMetError);
        }

        let settings = storage::get_settings(&e);
        if e.ledger().sequence() > proposal_data.eta + settings.grace_period {
            proposal_data.status = ProposalStatus::Expired;
            GovernorEvents::proposal_expired(&e, proposal_id);
        } else {
            let proposal_config = storage::get_proposal_config(&e, proposal_id).unwrap_optimized();
            proposal_config.execute(&e);
            proposal_data.status = ProposalStatus::Executed;
            GovernorEvents::proposal_executed(&e, proposal_id);
        }
        storage::set_proposal_data(&e, proposal_id, &proposal_data);
    }

    fn cancel(e: Env, from: Address, proposal_id: u32) {
        storage::extend_instance(&e);
        from.require_auth();

        let mut proposal_data = storage::get_proposal_data(&e, proposal_id)
            .unwrap_or_else(|| panic_with_error!(&e, GovernorError::NonExistentProposalError));

        // require from to be the creator or the council
        if from != proposal_data.creator {
            let council = storage::get_council_address(&e);
            if from != council {
                panic_with_error!(&e, GovernorError::UnauthorizedError);
            } else {
                // block the security council from canceling council proposals
                let proposal_config =
                    storage::get_proposal_config(&e, proposal_id).unwrap_optimized();
                match proposal_config.action {
                    ProposalAction::Council(_) => {
                        panic_with_error!(&e, GovernorError::UnauthorizedError);
                    }
                    _ => {}
                }
            }
        }

        if proposal_data.status != ProposalStatus::Open {
            panic_with_error!(&e, GovernorError::ProposalClosedError);
        }
        if proposal_data.vote_start <= e.ledger().sequence() {
            panic_with_error!(&e, GovernorError::ProposalVotePeriodStartedError);
        }
        proposal_data.status = ProposalStatus::Canceled;
        storage::set_proposal_data(&e, proposal_id, &proposal_data);
        storage::del_open_proposal(&e, &proposal_data.creator);
        GovernorEvents::proposal_canceled(&e, proposal_id);
    }

    fn vote(e: Env, voter: Address, proposal_id: u32, support: u32) {
        voter.require_auth();
        storage::extend_instance(&e);
        let proposal_data = storage::get_proposal_data(&e, proposal_id)
            .unwrap_or_else(|| panic_with_error!(&e, GovernorError::NonExistentProposalError));

        if proposal_data.status != ProposalStatus::Open {
            panic_with_error!(&e, GovernorError::ProposalClosedError);
        }
        if proposal_data.vote_start > e.ledger().sequence()
            || proposal_data.vote_end < e.ledger().sequence()
        {
            panic_with_error!(&e, GovernorError::OutsideOfVotePeriodError);
        }
        if storage::get_voter_support(&e, &voter, proposal_id).is_some() {
            panic_with_error!(&e, GovernorError::AlreadyVotedError);
        }

        let voter_power = VotesClient::new(&e, &storage::get_voter_token_address(&e))
            .get_past_votes(&voter, &proposal_data.vote_start);
        if voter_power <= 0 {
            panic_with_error!(&e, GovernorError::InsufficientVotingUnitsError);
        }

        let mut vote_count = storage::get_proposal_vote_count(&e, proposal_id).unwrap_optimized();
        vote_count.add_vote(&e, support, voter_power);

        storage::create_voter_support(&e, &voter, proposal_id, support);
        storage::set_proposal_vote_count(&e, proposal_id, &vote_count);

        GovernorEvents::vote_cast(&e, proposal_id, voter, support, voter_power);
    }

    fn get_vote(e: Env, voter: Address, proposal_id: u32) -> Option<u32> {
        storage::get_voter_support(&e, &voter, proposal_id)
    }

    fn get_proposal_votes(e: Env, proposal_id: u32) -> Option<VoteCount> {
        storage::get_proposal_vote_count(&e, proposal_id)
    }
}
#[contractimpl]
impl GovernorContract {
    pub fn update_proposal_threshold(env: Env) {
        let council = storage::get_council_address(&env);
        council.require_auth();
        let scf_token_client = VotesClient::new(&env, &storage::get_voter_token_address(&env));
        let target_threshold = scf_token_client.optimal_threshold();
        let mut settings = storage::get_settings(&env);
        settings.proposal_threshold = target_threshold;
        storage::set_settings(&env, &settings);
    }
}

#[cfg(test)]
mod test {
    use governance::LayerAggregator;
    use soroban_sdk::testutils::Address as AddressTrait;
    use soroban_sdk::{vec, Address, Env, Map, String, Vec, I256};

    use super::{GovernorContract, GovernorContractClient};
    use crate::constants::ONE_DAY_LEDGERS;
    use crate::settings::require_valid_settings;
    use crate::types::{GovernorSettings, ProposalAction};

    pub mod scf_token {
        use soroban_sdk::contractimport;
        contractimport!(file = "../target/wasm32-unknown-unknown/release/scf_token.wasm");
    }

    pub mod governance {
        use soroban_sdk::contractimport;
        contractimport!(file = "../target/wasm32-unknown-unknown/release/governance.wasm");
    }

    fn prepare_test(
        env: &Env,
        round: u32,
    ) -> (
        GovernorContractClient<'_>,
        governance::Client<'_>,
        scf_token::Client<'_>,
        Address,
    ) {
        env.cost_estimate().budget().reset_unlimited();
        let admin = Address::generate(&env);
        env.mock_all_auths();

        let governance_address = env.register(governance::WASM, ());
        let governance_client: governance::Client<'_> =
            governance::Client::new(&env, &governance_address);
        governance_client.initialize(&admin, &round);
        let neurons = soroban_sdk::vec![
            &env,
            (
                soroban_sdk::String::from_str(env, "Layer1"),
                I256::from_i128(env, 10_i128.pow(18)),
            ),
        ];
        governance_client.add_layer(&neurons, &LayerAggregator::Sum);

        let governor_address = env.register(GovernorContract, ());
        let governor_client: GovernorContractClient<'_> =
            GovernorContractClient::new(&env, &governor_address);

        let scf_token_address = env.register(scf_token::WASM, ());
        let scf_token_client: scf_token::Client<'_> =
            scf_token::Client::new(&env, &scf_token_address);
        scf_token_client.initialize(&admin, &governance_address);

        let settings = GovernorSettings {
            proposal_threshold: 10_000_000,
            vote_delay: ONE_DAY_LEDGERS,
            vote_period: ONE_DAY_LEDGERS * 5,
            timelock: ONE_DAY_LEDGERS,
            grace_period: ONE_DAY_LEDGERS * 7,
            quorum: 100,
            counting_type: 2,
            vote_threshold: 5100,
        };
        require_valid_settings(&env, &settings);
        governor_client.initialize(&scf_token_address, &admin, &settings);
        (governor_client, governance_client, scf_token_client, admin)
    }

    fn set_nqg_results(
        env: &Env,
        governance_client: &governance::Client,
        address: &Address,
        new_balance: i128,
    ) {
        let mut result = governance_client
            .try_get_neuron_result(
                &soroban_sdk::String::from_str(env, "0"),
                &soroban_sdk::String::from_str(env, "0"),
            )
            .unwrap_or_else(|_| {
                let mut map = Map::new(env);
                map.set(address.to_string(), I256::from_i32(env, 0));
                Ok(map)
            })
            .unwrap();
        result.set(address.to_string(), I256::from_i128(env, new_balance));

        governance_client.set_neuron_result(
            &soroban_sdk::String::from_str(env, "0"),
            &soroban_sdk::String::from_str(env, "0"),
            &result,
        );

        governance_client.calculate_voting_powers();
    }

    #[test]
    fn test_update_proposal_threshold() {
        let env = Env::default();
        let (governor_client, governance_client, scf_token_client, _admin) = prepare_test(&env, 30);

        let random_balances: Vec<i128> = vec![&env, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        for b in &random_balances {
            let addr = Address::generate(&env);
            set_nqg_results(&env, &governance_client, &addr, b * 10_i128.pow(18));
            scf_token_client.update_balance(&addr);
        }
        governor_client.update_proposal_threshold();
        let updated_settings = governor_client.settings();
        assert_eq!(updated_settings.proposal_threshold, 6 * 10_i128.pow(9));
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #214)")]
    fn disallow_council_proposal() {
        let env = Env::default();
        let (governor_client, governance_client, scf_token_client, _admin) = prepare_test(&env, 30);

        let creator = Address::generate(&env);
        set_nqg_results(&env, &governance_client, &creator, 10_i128.pow(18));
        scf_token_client.update_balance(&creator);
        governor_client.propose(
            &creator,
            &String::from_str(&env, "title"),
            &String::from_str(&env, "description"),
            &ProposalAction::Council(Address::generate(&env)),
        );
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #214)")]
    fn user_cant_propose_settings() {
        let env = Env::default();
        let (governor_client, governance_client, scf_token_client, _admin) = prepare_test(&env, 30);

        let random_user = Address::generate(&env);
        set_nqg_results(&env, &governance_client, &random_user, 10_i128.pow(18));
        scf_token_client.update_balance(&random_user);
        governor_client.propose(
            &random_user,
            &String::from_str(&env, "title"),
            &String::from_str(&env, "description"),
            &ProposalAction::Council(Address::generate(&env)),
        );
    }

    #[test]
    fn council_can_propose_settings() {
        let env = Env::default();
        let (governor_client, governance_client, scf_token_client, council) =
            prepare_test(&env, 30);

        set_nqg_results(&env, &governance_client, &council, 10_i128.pow(18));
        scf_token_client.update_balance(&council);

        let settings = GovernorSettings {
            proposal_threshold: 10_000_000,
            vote_delay: ONE_DAY_LEDGERS,
            vote_period: ONE_DAY_LEDGERS * 5,
            timelock: ONE_DAY_LEDGERS,
            grace_period: ONE_DAY_LEDGERS * 7,
            quorum: 100,
            counting_type: 2,
            vote_threshold: 5100,
        };
        governor_client.propose(
            &council,
            &String::from_str(&env, "title"),
            &String::from_str(&env, "description"),
            &ProposalAction::Settings(settings),
        );
    }
}
