# Governor 
This contract is from [Soroban Governor](https://github.com/script3/soroban-governor)
Some modifications were made to adapt it to our use case.
- Disabled council proposals.
- Settings proposals can only be created by council
- Added option to update proposal threshold without creating a _settings proposal_

### Updating proposal threshold
After each SCF voting round, voting powers of users change, and so do balances of scf_token. Because we always want top 10% (or at least 5) users to be able to create proposals, we have to update _proposal threshold_ setting in the Governor contract. To do this automatically, without creating a settings proposal, we have implemented an additional  ```update_proposal_threshold()``` function. It doesn't require any input data, making it safe and easy to use. Upon calling, it will make a cross-contract call to the scf_token contract to get the optimal threshold value and update its settings accordingly. This change goes into effect immediatly, without disturbing previously created proposals. 
The optimal threshold value is calculated on the scf_token contract side. [See scf_token docs for details](../scf_token/README.md)