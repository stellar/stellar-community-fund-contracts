use soroban_governor::{
    types::{Calldata, GovernorSettings, ProposalAction},
    GovernorContract, GovernorContractClient,
};
use soroban_sdk::{
    testutils::{Address as _, StellarAssetContract},
    token::{StellarAssetClient, TokenClient},
    vec, Address, Env, IntoVal, Map, String, Symbol, I256,
};

use crate::ONE_DAY_LEDGERS;

pub mod scf_token {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/scf_token.wasm");
}

pub mod governance {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/governance.wasm");
}

pub mod mock_subcall {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/mock_subcall.wasm");
}

pub use governance::Client as GovernanceClient;
pub use governance::LayerAggregator;
pub use mock_subcall::Client as MockSubcallClient;
pub use scf_token::Client as SCFTokenClient;

/// Register a `mock-subcall` instance pointing at `token` and `governor`.
pub fn create_mock_subcall_contract<'a>(
    e: &'a Env,
    token: &Address,
    governor: &Address,
) -> (Address, MockSubcallClient<'a>) {
    let address = e.register(mock_subcall::WASM, ());
    let client = MockSubcallClient::new(e, &address);
    client.initialize(token, governor);
    (address, client)
}

/// Register a Stellar Asset Contract (SEP-41 compliant). Used only by subcall/
/// calldata-execute tests where the proposal needs a transferable underlying asset;
/// `scf_token` does not support transfers by design.
///
/// Returns `(address, admin_client, token_client)`. The admin client is used for `mint`,
/// the token client for `transfer` / `balance`.
pub fn create_sep41_token<'a>(
    e: &'a Env,
    admin: &Address,
) -> (Address, StellarAssetClient<'a>, TokenClient<'a>) {
    let sac: StellarAssetContract = e.register_stellar_asset_contract_v2(admin.clone());
    let address = sac.address();
    let admin_client = StellarAssetClient::new(e, &address);
    let token_client = TokenClient::new(e, &address);
    (address, admin_client, token_client)
}

/// Default governance round used by `create_governor`.
pub const DEFAULT_ROUND: u32 = 30;

/// Default governor settings used by tests.
pub fn default_governor_settings() -> GovernorSettings {
    GovernorSettings {
        proposal_threshold: 10_000_000,
        vote_delay: ONE_DAY_LEDGERS,
        vote_period: ONE_DAY_LEDGERS * 7,
        timelock: ONE_DAY_LEDGERS,
        grace_period: ONE_DAY_LEDGERS * 7,
        quorum: 100,          // 1%
        counting_type: 2,     // 0x...010 (for)
        vote_threshold: 5100, // 51%
    }
}

/// Default proposal title/description/action (Calldata).
pub fn default_proposal_data(e: &Env) -> (String, String, ProposalAction) {
    let calldata = Calldata {
        contract_id: Address::generate(e),
        function: Symbol::new(e, "test"),
        args: (1, 2, 3).into_val(e),
        auths: vec![
            e,
            Calldata {
                contract_id: Address::generate(e),
                function: Symbol::new(e, "test"),
                args: (1, 2, 3).into_val(e),
                auths: vec![e],
            },
        ],
    };
    let title = String::from_str(e, "Test Title");
    let description = String::from_str(e, "# This is a cool proposal");

    (title, description, ProposalAction::Calldata(calldata))
}

/// Test fixture wiring up governance + `scf_token` + governor.
///
/// `admin` is also used as the governor council and as the `scf_token` / governance admin.
pub struct GovernorFixture<'a> {
    pub env: &'a Env,
    pub admin: Address,
    pub governor: GovernorContractClient<'a>,
    pub governor_address: Address,
    pub governance: GovernanceClient<'a>,
    pub token: SCFTokenClient<'a>,
    pub token_address: Address,
}

impl GovernorFixture<'_> {
    /// Whitelist `addrs` as proposal creators.
    pub fn whitelist(&self, addrs: &[Address]) {
        let mut list = soroban_sdk::Vec::new(self.env);
        for a in addrs {
            list.push_back(a.clone());
        }
        self.governor.update_proposal_whitelist(&list);
    }

    /// Set `addr`'s NQG-derived `scf_token` balance to `balance` (in `scf_token` decimals — 10^9).
    ///
    /// Sets the neuron result, recalculates voting powers, then calls `update_balance`.
    /// May only be called once per round per address.
    pub fn set_voter_balance(&self, addr: &Address, balance: i128) {
        // scf_token divides NQG by 10^9 to get its balance.
        let nqg_value =
            I256::from_i128(self.env, balance).mul(&I256::from_i32(self.env, 10).pow(9));

        let mut result = self
            .governance
            .try_get_neuron_result(
                &String::from_str(self.env, "0"),
                &String::from_str(self.env, "0"),
            )
            .unwrap_or_else(|_| Ok(Map::new(self.env)))
            .unwrap();
        result.set(addr.to_string(), nqg_value);

        self.governance.set_neuron_result(
            &String::from_str(self.env, "0"),
            &String::from_str(self.env, "0"),
            &result,
        );
        self.governance.calculate_voting_powers();

        self.token.update_balance(addr);
    }
}

/// Build a fresh fixture: deploys governance, `scf_token`, and the governor with default-ish wiring.
///
/// `admin` is the governance/token admin AND the governor council.
pub fn create_governor<'a>(
    e: &'a Env,
    admin: &Address,
    council: &Address,
    settings: &GovernorSettings,
) -> GovernorFixture<'a> {
    e.cost_estimate().budget().reset_unlimited();
    e.mock_all_auths();

    let governance_address = e.register(governance::WASM, (admin.clone(), DEFAULT_ROUND));
    let governance_client = GovernanceClient::new(e, &governance_address);
    let neurons = vec![
        e,
        (
            String::from_str(e, "Layer1"),
            I256::from_i128(e, 10_i128.pow(18)),
        ),
    ];
    governance_client.add_layer(&neurons, &LayerAggregator::Sum);

    let token_address = e.register(scf_token::WASM, (admin.clone(), governance_address.clone()));
    let token_client = SCFTokenClient::new(e, &token_address);

    let governor_address = e.register(
        GovernorContract,
        (token_address.clone(), council.clone(), settings.clone()),
    );
    let governor_client = GovernorContractClient::new(e, &governor_address);

    GovernorFixture {
        env: e,
        admin: admin.clone(),
        governor: governor_client,
        governor_address,
        governance: governance_client,
        token: token_client,
        token_address,
    }
}
