use soroban_sdk::testutils::{Address as AddressTrait, MockAuth, MockAuthInvoke};
use soroban_sdk::{Address, Env, IntoVal};

use crate::e2e::common::contract_utils::{deploy_and_setup, Deployment};

#[test]
fn transfer_admin() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let Deployment {
        client, address, ..
    } = deploy_and_setup(&env, &admin);

    // Call authorized function
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "update_balance",
            args: (&address,).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.update_balance(&address);

    // Transfer admin
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "transfer_admin",
            args: (&new_admin,).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.transfer_admin(&new_admin);

    // Verify old admin can no longer call contract
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "transfer_admin",
            args: (&new_admin,).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    assert!(client.try_transfer_admin(&new_admin).is_err());

    // Verify new admin cal call contract
    env.mock_auths(&[MockAuth {
        address: &new_admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "update_balance",
            args: (&address,).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.update_balance(&address);
}
