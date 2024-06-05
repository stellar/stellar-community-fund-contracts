#![no_main]

use crate::arbitrary::Unstructured;
use governance::{LayerAggregator, VotingSystem, VotingSystemClient};
use libfuzzer_sys::fuzz_target;
use scf_token::{SCFToken, SCFTokenClient};
use soroban_sdk::testutils::arbitrary::{arbitrary, fuzz_catch_panic, Arbitrary};
use soroban_sdk::testutils::Address as AddressTrait;
use soroban_sdk::{Address, Env, Map, I256};

pub fn deploy_contract<'a>(
    env: &Env,
    governance_address: &Address,
    admin: &Address,
) -> SCFTokenClient<'a> {
    let scf_token_address = env.register_contract(None, SCFToken);
    let scf_token_client = SCFTokenClient::new(env, &scf_token_address);

    scf_token_client.initialize(admin, governance_address);

    scf_token_client
}

pub fn deploy_scf_contract<'a>(env: &Env, admin: &Address) -> VotingSystemClient<'a> {
    let governance_address = env.register_contract(None, VotingSystem);
    let governance_client = VotingSystemClient::new(env, &governance_address);

    governance_client.initialize(admin, &25);

    let neurons = soroban_sdk::vec![
        &env,
        (
            soroban_sdk::String::from_str(env, "Layer1"),
            I256::from_i128(env, 10_i128.pow(18)),
        ),
    ];
    governance_client.add_layer(&neurons, &LayerAggregator::Sum);

    governance_client
}

pub struct Deployment<'a> {
    pub client: SCFTokenClient<'a>,
    pub governance_client: VotingSystemClient<'a>,
    pub address: Address,
}

pub fn deploy_and_setup<'a>(env: &Env, admin: &Address) -> Deployment<'a> {
    env.mock_all_auths();

    let governance_client = deploy_scf_contract(env, admin);
    let client = deploy_contract(env, &governance_client.address, admin);

    let address = Address::generate(env);

    let mut result = Map::new(env);
    result.set(address.to_string(), I256::from_i128(env, 10_i128.pow(18)));

    governance_client.set_neuron_result(
        &soroban_sdk::String::from_str(env, "0"),
        &soroban_sdk::String::from_str(env, "0"),
        &result,
    );

    governance_client.calculate_voting_powers();

    env.budget().reset_default();
    client.update_balance(&address);

    env.set_auths(&[]);

    Deployment {
        client,
        governance_client,
        address,
    }
}

pub fn update_balance(
    env: &Env,
    client: &SCFTokenClient,
    governance_client: &VotingSystemClient,
    address: &Address,
    new_balance: i128,
) {
    let mut result = governance_client.get_neuron_result(
        &soroban_sdk::String::from_str(env, "0"),
        &soroban_sdk::String::from_str(env, "0"),
    );
    result.set(address.to_string(), I256::from_i128(env, new_balance));

    governance_client.set_neuron_result(
        &soroban_sdk::String::from_str(env, "0"),
        &soroban_sdk::String::from_str(env, "0"),
        &result,
    );

    governance_client.calculate_voting_powers();

    env.budget().reset_default();
    client.update_balance(address);
}

#[derive(Arbitrary, Debug)]
struct Input {
    #[arbitrary(with = |u: &mut Unstructured| u.int_in_range((i64::MIN as i128)..=(i64::MAX as i128)))]
    balance1: i128,
    #[arbitrary(with = |u: &mut Unstructured| u.int_in_range((i64::MIN as i128)..=(i64::MAX as i128)))]
    balance2: i128,
}

fuzz_target!(|input: Input| {
    let env = Env::default();
    let admin = Address::generate(&env);

    let Deployment {
        client,
        governance_client,
        address,
    } = deploy_and_setup(&env, &admin);
    let address2 = Address::generate(&env);
    env.mock_all_auths();

    {
        let _ = fuzz_catch_panic(|| {
            update_balance(
                &env,
                &client,
                &governance_client,
                &address,
                input.balance1 * 10_i128.pow(18),
            );
            update_balance(
                &env,
                &client,
                &governance_client,
                &address2,
                input.balance2 * 10_i128.pow(18),
            );
        });

        assert_eq!(client.get_votes(&address), input.balance1 * 10_i128.pow(9));
        assert_eq!(client.get_votes(&address2), input.balance2 * 10_i128.pow(9));
        assert_eq!(
            client.total_supply(),
            (input.balance1 + input.balance2) * 10_i128.pow(9)
        );
    }
});
