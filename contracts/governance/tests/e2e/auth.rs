use crate::e2e::common::contract_utils::deploy_contract_without_initialization;
use soroban_sdk::testutils::{
    Address as AddressTrait, AuthorizedFunction, AuthorizedInvocation, MockAuth, MockAuthInvoke,
};
use soroban_sdk::xdr::{ScErrorCode, ScErrorType};
use soroban_sdk::{vec, Address, Env, Error, IntoVal, String, Symbol};

#[test]
fn uninitialized_contract_is_not_callable() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_client = deploy_contract_without_initialization(&env);

    let result = contract_client.try_set_current_round(&25);
    assert_eq!(
        result,
        Err(Ok(Error::from_type_and_code(
            ScErrorType::Context,
            ScErrorCode::InvalidAction
        )))
    );
}

#[test]
fn auth() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_client = deploy_contract_without_initialization(&env);

    let admin = Address::generate(&env);
    contract_client.initialize(&admin, &25);

    let submission_name = String::from_str(&env, "abc");
    contract_client.set_submissions(&vec![
        &env,
        (submission_name.clone(), String::from_str(&env, "Applications")),
    ]);

    dbg!(&env.auths());

    assert_eq!(
        env.auths(),
        std::vec![(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    contract_client.address.clone(),
                    Symbol::new(&env, "set_submissions"),
                    (vec![&env, submission_name.clone()],).into_val(&env)
                )),
                sub_invocations: std::vec![],
            }
        ),]
    );
}

#[test]
fn transfer_admin() {
    let env = Env::default();
    let contract_client = deploy_contract_without_initialization(&env);

    let admin = Address::generate(&env);
    contract_client.initialize(&admin, &25);

    // Transfer admin
    let new_admin = Address::generate(&env);
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &contract_client.address,
            fn_name: "transfer_admin",
            args: (&new_admin,).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    contract_client.transfer_admin(&new_admin);

    // Verify old admin can no longer modify state
    let submission_name = String::from_str(&env, "abc");
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &contract_client.address,
            fn_name: "add_submission",
            args: (submission_name.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    let result = contract_client.try_set_submissions(&vec![
        &env,
        (submission_name.clone(), String::from_str(&env, "Applications")),
    ]);
    assert!(result.is_err());

    // Verify new admin can modify state
    env.mock_auths(&[MockAuth {
        address: &new_admin,
        invoke: &MockAuthInvoke {
            contract: &contract_client.address,
            fn_name: "set_submissions",
            args: (vec![&env, submission_name.clone()],).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    contract_client.set_submissions(&vec![
        &env,
        (submission_name.clone(), String::from_str(&env, "Applications")),
    ]);
}

#[test]
#[should_panic(expected = "Error(WasmVm, InvalidAction)")]
fn set_admin_again_panics() {
    let env = Env::default();
    let contract_client = deploy_contract_without_initialization(&env);

    let admin = Address::generate(&env);
    contract_client.initialize(&admin, &25);
    contract_client.initialize(&admin, &25);
}
