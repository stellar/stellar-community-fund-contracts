# Governor 
This contract is from [Soroban Governor](https://github.com/script3/soroban-governor)
Some modifications were made to adapt it to our use case.
- Disabled council proposals.
- Settings proposals can only be created by council
- Added proposal creation whitelist

### Updating proposal creation whitelist
To ensure annonimity of SCF voters creating proposals for their own projects, we don't allow creating proposals using the same address as used for voting. Instead after each SCF round, a new list of all pilots secondary addresses is generated using data from our sanity, and uploaded to this contract.