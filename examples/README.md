# How to use SCF's voting setup

This guide shows how to setup and run a similar voting system as SCF uses for community votes. It uses actual smart contracts, but everything else is a minimal working setup, just to demonstrate the voting flow. In the real world we use much more complicated setup with databases, full backend, and neurons compiled to WASM to work in js env.

## Neurons 
Neurons are used to calculate component values of voting power.
We input some data into each neuron, and it outputs some numeric value.
Then the results of all neurons are be uploaded to Neural Quorum Governance contract, which calculates the final voting power, according to how it was set up.

Why not upload all data into the contract and calculate all values there?
Doing so would be beneficial for transparency of the whole voting system, but comes with 2 problems:
 - privacy - some of the data we use for neurons would allow bad actors to link specific users to publickeys.
 - performance - due to high amout of data, and high complexity of calculations doing everything on-chain would not be possible in a reasonable time and cost.

### Minimal neurons setup
In `/examples/minimal-neurons` you can see a rust project that contains example neurons, along with the code that will trigger them. In the `data` folder there is some example data that will be used by the neurons as an input. For simplicity we will use data for a single voter. If you have rust installed, head into the correct folder and run:

`cd examples/minimal-neurons`
`cargo run`

In the `output` folder you'll see one output file for each neuron.

Neurons can perform any type of calculations, for example provide 0.5 points bonus for each round a user have participaded in, or something more complicated.
Now that we have the neurons results we can move on to the on-chain part of the system.

## Contracts 
Neural Quorum Governance contract is used to calculate voters voting powers (based on supplied neurons results), and tally the votes. Source code for it is located in `/contracts/governance`

### Deploying and initializing the contract 
First in the `examples` folder create a `.env` file, by changing the name of `env.example` and filling in your account data. Then simply run:

`./examples/scripts/deploy_governance.sh`

The script will compile, deploy, and initialize the contract, with your account as an admin, and setup the neural layer. 

### Uploading submissions
Submissions are projects names on which users will vote for or against.

// TODO - uploading submissions

### Uploading votes

// TODO - uploading votes

### Uploading neurons results

// TODO - uploading neurons results to contract

### Calculating voting powers
is done simply by triggering a function on the contract. Do it by running another script:

`./examples/scripts/calculate_voting_powers.sh`

### Tallying the votes

## Storing voting power as smart contract token value

### Deploy SCF token contract 

### Fetch voting powers from NQG contract