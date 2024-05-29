use crate::e2e::common::contract_utils::{deploy_and_setup, Deployment};
use soroban_sdk::testutils::Address as AddressTrait;
use soroban_sdk::xdr::{ScErrorCode, ScErrorType};
use soroban_sdk::{Address, Env, Error, String};

#[test]
fn allowance() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let Deployment {
        client, address, ..
    } = deploy_and_setup(&env, &admin);

    assert_eq!(
        client.try_allowance(&address, &Address::generate(&env)),
        Err(Ok(Error::from_type_and_code(
            ScErrorType::Context,
            ScErrorCode::InvalidAction
        )))
    );
}

#[test]
fn approve() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let Deployment {
        client, address, ..
    } = deploy_and_setup(&env, &admin);

    assert_eq!(
        client.try_approve(
            &address,
            &Address::generate(&env),
            &1,
            &(&env.ledger().sequence() + 100)
        ),
        Err(Ok(Error::from_type_and_code(
            ScErrorType::Context,
            ScErrorCode::InvalidAction
        )))
    );
}

#[test]
fn balance() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let Deployment {
        client, address, ..
    } = deploy_and_setup(&env, &admin);

    assert_eq!(client.balance(&address), 1);
}

#[test]
fn transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let Deployment {
        client, address, ..
    } = deploy_and_setup(&env, &admin);

    assert_eq!(
        client.try_transfer(&address, &Address::generate(&env), &1,),
        Err(Ok(Error::from_type_and_code(
            ScErrorType::Context,
            ScErrorCode::InvalidAction
        )))
    );
}

#[test]
fn transfer_from() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let Deployment {
        client, address, ..
    } = deploy_and_setup(&env, &admin);

    assert_eq!(
        client.try_transfer_from(&address, &address, &Address::generate(&env), &1,),
        Err(Ok(Error::from_type_and_code(
            ScErrorType::Context,
            ScErrorCode::InvalidAction
        )))
    );
}

#[test]
fn burn() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let Deployment {
        client, address, ..
    } = deploy_and_setup(&env, &admin);

    assert_eq!(
        client.try_burn(&address, &1,),
        Err(Ok(Error::from_type_and_code(
            ScErrorType::Context,
            ScErrorCode::InvalidAction
        )))
    );
}

#[test]
fn burn_from() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let Deployment {
        client, address, ..
    } = deploy_and_setup(&env, &admin);

    assert_eq!(
        client.try_burn_from(&address, &address, &1,),
        Err(Ok(Error::from_type_and_code(
            ScErrorType::Context,
            ScErrorCode::InvalidAction
        )))
    );
}

#[test]
fn decimals() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let Deployment {
        client, address, ..
    } = deploy_and_setup(&env, &admin);

    assert_eq!(client.decimals(), 0);
}

#[test]
fn name() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let Deployment {
        client, address, ..
    } = deploy_and_setup(&env, &admin);

    assert_eq!(client.name(), String::from_str(&env, "NQG Token"));
}

#[test]
fn symbol() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let Deployment {
        client, address, ..
    } = deploy_and_setup(&env, &admin);

    assert_eq!(client.symbol(), String::from_str(&env, "NQG"));
}
