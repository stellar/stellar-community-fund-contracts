# Stellar Community Fund Token

This is a Token contract, and Votes contract for Soroban Governor.

It implements the [token interface](https://developers.stellar.org/docs/tokens/token-interface). Balance can be seen for example in Freighter wallet. _It is not a Stellar asset wrapper_
It's non-transferable, can't be burned or minted, it's balances are set to users voting power from the [Governance Contract](contracts/governance/README.md) after each voting round.
Simultaneously it implements [votes interface](https://github.com/script3/soroban-governor/blob/main/contracts/votes/src/votes.rs) required by the [Soroban Governor](contracts/governor/README.md) to allow usage of a token as votes in the DAO.

### Optimal proposal threshold calculation
This functionality is used by the [Governor](contracts/governor/README.md) contract.
```optimal_threshold()``` works by reading addresses of all users from storage, reading balance for each user and sorting those balances. 
Returned value is the smallest balance from the top 10%. If the top 10% is less than 5 users, the 5th highest balance is returned. 
This way always 10% of users with highest balance can create proposals, but never less than 5 people. 

Additionally before calculating the result, contract checks if all users have their balances updated from the same round. Given that ```update_balance()``` function requires address of a user as an argument, and is called from an external script, if something goes wrong, it is possible that not all users will have their balance updated to the most recent round.