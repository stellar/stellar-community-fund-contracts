# Neural Quorum Governance Contracts
> ⚠️ Code in this repository has not been audited and is under development.

[Neural Quorum Governance](https://stellarcommunityfund.gitbook.io/scf-handbook/community-involvement/governance/neural-quorum-governance) is a governance framework implemented on the Stellar blockchain.
This repository contains smart contract used to conduct voting on-chain and rust modules we are using to calculate
neurons voting powers, as well as some additional helper modules for testing and development.

## `/neurons`

Contains source code of neurons. [See neurons docs for more details](neurons/README.md).

## `/contracts`

Contains the source code of various smart contracts:

#### `governance`
Neural Quorum Governance contract. [See contract docs for more details](contracts/governance/README.md).

#### `scf_token`
Stellar Community Fund Token contract. [See contract docs for more details](contracts/scf_token/README.md).

#### `governor`
[Soroban Governor](https://github.com/script3/soroban-governor) contract, slightly modified for our specific use case. [See contract docs for more details](contracts/governor/README.md).

## `/offchain`
Contains source code of additional utility module. [See offchain docs for more details](offchain/README.md).

## Reporting Bugs and Issues

Found a bug or an issue and want to report it?
Please [open an issue](https://github.com/stellar/stellar-community-fund-contracts/issues).

## Contributing

This repository is under active development. Read the [contributing guidelines](./CONTRIBUTING.md) for more
information.
