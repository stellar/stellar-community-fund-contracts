use soroban_sdk::{Address, Env};

/// This was copied over from https://github.com/script3/soroban-governor as they do not expose
/// this trait as a dependency. 
pub trait Votes {
    /// Get the total supply of voting tokens
    fn total_supply(e: Env) -> i128;

    /// Set a new sequence number of a future vote. This ensures vote history is maintained
    /// for old votes.
    ///
    /// Requires auth from the governor contract
    ///
    /// ### Arguments
    /// * `sequence` - The sequence number of the vote
    fn set_vote_sequence(e: Env, sequence: u32);

    /// Get the total supply of voting tokens at a specific ledger sequence number.
    /// The ledger must be finalized before the sequence number can be used.
    ///
    /// ### Arguments
    /// * `sequence` - The sequence number to get the total voting token supply at
    ///
    /// ### Panics
    /// Panics if the sequence number is greater than or equal to the current ledger sequence.
    fn get_past_total_supply(e: Env, sequence: u32) -> i128;

    /// Get the current voting power of an account
    ///
    /// ### Arguments
    /// * `account` - The address of the account
    fn get_votes(e: Env, account: Address) -> i128;

    /// Get the voting power of an account at a specific ledger sequence number.
    /// The ledger must be finalized before the sequence number can be used.
    ///
    /// ### Arguments
    /// * `account` - The address of the account
    /// * `sequence` - The sequence number to get the voting power at
    ///
    /// ### Panics
    /// Panics if the sequence number is greater than or equal to the current ledger sequence.
    fn get_past_votes(e: Env, user: Address, sequence: u32) -> i128;

    /// Get the deletage that account has chosen
    ///
    /// ### Arguments
    /// * `account` - The address of the account
    fn get_delegate(e: Env, account: Address) -> Address;

    /// Delegate the voting power of the account to a delegate
    ///
    /// ### Arguments
    /// * `delegate` - The address of the delegate
    fn delegate(e: Env, account: Address, delegatee: Address);
}
