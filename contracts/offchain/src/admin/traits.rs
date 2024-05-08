use soroban_sdk::{Address, BytesN, Env};

pub trait Admin {
    /// Transfer ownership of the contract to `new_admin` address.
    fn transfer_admin(env: Env, new_admin: Address);
    /// Upgrade the implementation of the contract with one identified by `wasm_hash`.
    fn upgrade(env: Env, wasm_hash: BytesN<32>);
}
