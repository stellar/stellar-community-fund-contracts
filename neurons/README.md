# Neurons

This code is an implementation of `neurons`
from the [Neural Quorum Governance](https://stellarcommunityfund.gitbook.io/module-library).
The data computed by this package is uploaded to the voting contract.

This is the source code used to calculate voting powers for each neuron used in NQG mechanism.
It also normalized the votes of voters (converts delegations to final votes) for them to be uploaded to the contract.

## Inputs

Neurons expect the inputs to be provided in `json` format. The inputs are loaded from `data/` directory.

## Outputs

Computed voting powers and normalized votes are written to `result/` directory.

## Running

```shell
cargo run
```

## Neurons

### Assigned Reputation Neuron

Assigns voting power based on voter discord rank.

### Prior Voting History Neuron

Assigns voting power based on rounds voter previously participated in.

### Trust Graph Neuron

Assigns voting power based on trust assigned to voter by other voters.
It uses min-max normalized PageRank algorithm to compute the score.

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
