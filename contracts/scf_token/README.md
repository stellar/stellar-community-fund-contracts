# Stellar Community Fund Token

This is a Token contract, and Votes contract for Soroban Governor.

It implements the [token interface](https://developers.stellar.org/docs/tokens/token-interface). Balance can be seen for example in Freighter wallet. _It is not a Stellar asset wrapper_
It's non-transferable, can't be burned or minted, it's balances are set to users voting power from the [Governance Contract](contracts/governance/README.md) after each voting round.
Simultaneously it implements [votes interface](https://github.com/script3/soroban-governor/blob/main/contracts/votes/src/votes.rs) required by the [Soroban Governor](contracts/governor/README.md) to allow usage of a token as votes in the DAO.