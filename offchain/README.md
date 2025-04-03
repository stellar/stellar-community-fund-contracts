# Offchain

This module allows running the Neural Governance contract locally.
It's useful for developing and testing new features without constantly re-compiling to WASM and deploying to the network.

### main.rs
Entry point

### data_generator.rs
Takes json data from Neurons and converts it to SorobanSDK data types, allowing direct input of data to contract functions.

### offchain.rs
Prepares and calls contract functions with data from data_generator.rs