use soroban_sdk::testutils::{Address as AddressTrait, MockAuth, MockAuthInvoke};
use soroban_sdk::{Address, Env, IntoVal};

use crate::e2e::common::contract_utils::{
    bump_round, deploy_and_setup, governance, jump, set_nqg_results, Deployment,
};

fn authorized_wrapper<F>(env: &Env, f: F)
where
    F: Fn(),
{
    env.mock_all_auths();

    f();

    env.set_auths(&[]);
}

fn authorized_set_nqg_results(
    env: &Env,
    governance_client: &governance::Client,
    address: &Address,
    new_balance: i128,
) {
    authorized_wrapper(env, || {
        set_nqg_results(env, governance_client, address, new_balance);
    });
}

fn authorized_bump_round(env: &Env, governance_client: &governance::Client) {
    authorized_wrapper(env, || bump_round(governance_client));
}

#[test]
fn transfer_admin() {
    let mut env = Env::default();

    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let Deployment {
        client,
        governance_client,
        ..
    } = deploy_and_setup(&env, &admin);

    let address = Address::generate(&env);
    authorized_set_nqg_results(&env, &governance_client, &address, 10_i128.pow(18));

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

    authorized_bump_round(&env, &governance_client);

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

    jump(&mut env, 1);
    authorized_bump_round(&env, &governance_client);
    authorized_set_nqg_results(&env, &governance_client, &address, 2 * 10_i128.pow(18));

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
