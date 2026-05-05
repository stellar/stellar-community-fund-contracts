use soroban_sdk::testutils::{
    Address as AddressTrait, AuthorizedFunction, AuthorizedInvocation, MockAuth, MockAuthInvoke,
};
use soroban_sdk::{vec, Address, Env, IntoVal, Symbol};

use crate::e2e::common::contract_utils::deploy_contract;

#[test]
fn auth() {
    let env = Env::default();
    let (contract_client, admin) = deploy_contract(&env);
    env.mock_all_auths();

    contract_client.set_current_round(&30);
    assert_eq!(
        env.auths(),
        std::vec![(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    contract_client.address.clone(),
                    Symbol::new(&env, "set_current_round"),
                    vec![&env, 30_u32.into()]
                )),
                sub_invocations: std::vec![],
            }
        ),]
    );
}

#[test]
fn transfer_admin() {
    let env = Env::default();
    let (contract_client, admin) = deploy_contract(&env);

    // Transfer admin
    let new_admin = Address::generate(&env);
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &contract_client.address,
            fn_name: "transfer_admin",
            args: vec![&env, new_admin.into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    contract_client.transfer_admin(&new_admin);

    // Verify old admin can no longer modify state
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &contract_client.address,
            fn_name: "set_current_round",
            args: vec![&env, 30_u32.into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    let result = contract_client.try_set_current_round(&30_u32);
    assert!(result.is_err());

    // Verify new admin can modify state
    env.mock_auths(&[MockAuth {
        address: &new_admin,
        invoke: &MockAuthInvoke {
            contract: &contract_client.address,
            fn_name: "set_current_round",
            args: vec![&env, 30_u32.into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    let result = contract_client.try_set_current_round(&30_u32);
    assert!(result.is_ok());
}
