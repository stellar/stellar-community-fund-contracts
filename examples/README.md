# How to use SCF's Implementation of Neural Quorum Governance

This guide shows how to setup and run a similar voting system as SCF uses for Community Vote. It uses actual Soroban smart contracts, but everything else is a minimal working setup, just to demonstrate the voting flow. In the actual implementation we use much more complicated setup with databases, full backend, and neurons compiled to WASM to work in js env. For this tutorial we'll use json files instead of actual database. Example files with data are located in `data` folder.

`cd examples` to make sure all scripts work correctly

## Neurons 
Neurons are used to calculate component values of voting power. We input some data into each neuron, and it outputs a numeric value. Output values are converted to fixed point decimal values to ensure no precission loss while converting data between different formats. Then the results of all neurons have to be uploaded to the Neural Quorum Governance contract, which will use this data to calculate the final voting power.

Why not upload all data into the contract and calculate all values there?
Doing so would be beneficial for transparency of the whole voting system, but comes with 2 problems:
 - privacy - some of the data we use for neurons would allow bad actors to link specific users to publickeys.
 - performance - due to high amount of data, and high complexity of calculations doing everything on-chain would not be possible in a reasonable time and cost.

### Minimal neurons setup
In `/neurons` you can see a rust project that contains example neurons, along with the code that will trigger them. In the `data` folder there is some example data that will be used by the neurons as an input. Neurons can perform any type of calculations, for example provide 0.5 points bonus for each round a user have participated in, or something more complicated. In this example we have 3 neurons, called Neuron1 Neuron2 and Neuron3. 

Neuron1: Multiplies input value by 1.5
Neuron2: Subtracts 20% from the input value
Neuron3: Multiplies input value by 3

If you have rust installed, head into the correct folder and run:

`pushd neurons`
`cargo run`
`popd`

In the `data` folder you'll see `neurons_output.json`. Now that we have the neurons results we can move on to the on-chain part of the system.

## NQG Contract 
Neural Quorum Governance contract is used to calculate voters voting powers (based on supplied neurons results), and tally the votes. Source code for it is located in `/contracts/governance`

### Deploying and initializing the contract 
First in the `examples` folder create a `.env` file, by changing the name of `env.example` and filling in your account data. This script will compile, deploy and initialize NQG contract. Also Layers will be setup (more about layers see `Uploading neurons results`) After deployment address of the contract will be saved in .env file for future use.

`./scripts/governance_deploy.sh`

The script will compile, deploy, and initialize the contract, with your account as an admin, and setup the neural layer. 

### Uploading submissions
Submissions are projects names on which users will vote for or against. A list of submissions is uploaded to the contract using the script below
`./scripts/governance_upload_submissions.sh`

### Uploading votes
Votes are uploaded separately for each submission. You provide submission name and a map of voters publickeys and votes.
`./scripts/governance_upload_votes.sh`

### Uploading neurons results
Neurons results need to be uploaded to allow calculating the final NQG score (voting power) on-chain. Each neuron is assigned to a "layer" and has its own id. For example we have 3 neurons, each assigned to a specific neuron in the contract.
Layer 1: neuron 0 and 1
Layer 2: neuron 0
Each Layer calculates its output based on how it was configured during deployment. In our examples we configured first layer as "Sum" - it means values from all neurons on this layer will be summed together, and Layer 2 as Product - values from neurons will be multiplied, although we have only one neuron on layer 2 so its output will be equal to the neuron value.
Results of each neuron, calculated beforehand will now be uploaded to the contract to its corresponding layer and neuron id.
`./scripts/governance_upload_neurons_results.sh`

### Calculating voting powers
After uploading neurons, we can trigger the calculation of voting powers. This function will run all of the Layers and save the results in storage, so it can be accessed anytime.

`./scripts/governance_calculate_voting_powers.sh`

### Tallying the votes
Now all data required to run the voting round is ready, we can tally the votes. It has to be done separately for each submission. Script showcases how to do it. Results for each submission is also saved in contract storage so it can be accessed later, for example to display it on the website, directly from on-chain data.

`./scripts/governance_tally_votes.sh`

### Setting the round number
After voting is complete, it is possible to change the round number on the contract, and use it for the next vote, without impacting previously saved data. This script will set current round on the contract to the one specified in .env file, also it will fetch current round from contract to verify round was correctly set.

`./scripts/governance_set_round.sh`

## (Optional) Storing voting power as smart contract token value
Voting powers calculated for the voting round can be easily used as a token, for example to use as voting power in DAO systems. For this we have our SCF Token smart contract. Source code is located in `/contracts/scf_token`. This contract will directly  call our previously deployed NQG contract to get voting powers of voters. Each user can have a balance of the token set to their voting power. Token contract we use also implements `votes` trait so it can be used as votes token in dao systems like Soroban Governor.

### Deploy SCF token contract 
First deploy and initialize the Token contract. Keep in mind that it needs address of the NQG contract to initialize, so make sure correct address is present in the .env file.
`./scripts/token_deploy.sh`

### Fetch voting powers from NQG contract
Now we can set token balances. Do this by calling a `update_balance()` function, and providing target user publickey. Token contract will trigger `get_voting_powers()` on the Neural Governance contract, and set token balance of this user equal to his voting power from NQG contract. It doesn't set balance the of all users automatically, since you may want to allow only some group of voters to have the token.
`./scripts/token_update_balance.sh`

### Check total votes distribution
To check total amount of votes distributed call `total_supply()`.
`./scripts/token_total_supply.sh`

## What's next
Now you have working bare-minimum setup of the Neural Quorum Governance Voting system. You can create a backend service that will nicely connect all of those elements into one api, however it suits your project, using one of many stellar sdk's
