# Neurons

This code is an implementation of `neurons`
from the [Neural Quorum Governance](https://stellarcommunityfund.gitbook.io/module-library). 
There are some additional mechanisms added after the initial implementation:
- Trust Graph Neuron additionaly takes into account trust score gain or loss between previous and current round.
- Prior Voting History Neuron uses logistic function to determine the bonus, instead of original linear solution. (right now users who were active more recently get bigger bonus)

The data computed by this package is uploaded to the voting contract.

This is the source code used to calculate voting powers for each neuron used in NQG mechanism.
It also normalized the votes of voters (converts delegations to final votes) for them to be uploaded to the contract.

## Inputs

Neurons expect the inputs to be provided in `json` format. The inputs are loaded from `data/` directory.

## Outputs

Computed results of each neuron and normalized votes are written to `result/` directory.

## Running

```shell
cargo run
```

## Neurons

### Assigned Reputation Neuron

Assigns voting power based on voter discord rank.

### Prior Voting History Neuron

Assigns voting power based on rounds voter previously participated in. 
Having participated in most recent rounds have greater impact on the bonus, than rounds that were long time ago.
This way users who are inactive slowly loose their bonus, allowing for new, more active users to catch up.

// TODO add curves and math formula here 

### Trust Graph Neuron
// TODO update to actual values after deciding on them

Assigns voting power based on the amount of users trusting given voter. Each user can select other voters as trusted.
Final trust score calculation is done in 3 steps:
1. min-max normalized PageRank algorithm to compute the initial score.
2. voters trusted by users considered highly trusted (top **1%** of scores from page rank) get additional bonus of **0.1** point
3. after steps 1 & 2 are done for current and previous round, difference between them is calculated. If voter lost more than **X%** of trust since previous round, **some** points are subtracted from his final score. This mechanism creates a penalty system for users who possibly did something very wrong and community doesn't consider them trustworthy anymore.

![trust graph neuron logic](./images/trust_neuron_logic.png)
## Development

### Running Tests

```shell
cargo test
```

### Running Lint

```shell
cargo lint
```

This is an alias to `cargo clippy` with special config. See `.cargo/config.toml` for more details.

### Formatting

```shell
cargo fmt
```
