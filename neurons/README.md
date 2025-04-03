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

### Trust Graph Neuron

Assigns voting power based on trust assigned to voter by other voters.
It uses min-max normalized PageRank algorithm to compute the score, and looks at the diference between score from previous and current round.
If loss of trust since the last round is big enough, curve of trust bonus loss gets steeper. This mechanism creates a penalty system for users who possibly did something very wrong and community doesn't consider them trustworthy anymore.

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
