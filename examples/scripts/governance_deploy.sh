#!/bin/bash
# Script builds, deploys, and initializes neural governance contract
ENV_PATH=".env"
source $ENV_PATH

echo "CURRENT_ROUND: $CURRENT_ROUND"

echo STEP 1: Build all contracts
pushd "../contracts"
  stellar contract build 
popd

echo STEP 2: Deploy governance contract
NEURAL_GOVERNANCE_ADDRESS=$(stellar contract deploy \
  --network $STELLAR_NETWORK \
  --wasm ../contracts/target/wasm32v1-none/release/governance.wasm \
  --rpc-url $STELLAR_RPC_URL \
  --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
  --source-account $STELLAR_SECRET_KEY)

echo "NEURAL_GOVERNANCE_ADDRESS: $NEURAL_GOVERNANCE_ADDRESS"

echo STEP 3: Initialize governance contract
stellar contract invoke \
  --id $NEURAL_GOVERNANCE_ADDRESS \
  --source-account $STELLAR_SECRET_KEY \
  --rpc-url $STELLAR_RPC_URL \
  --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
  -- initialize \
  --admin=$STELLAR_PUBLIC_KEY \
  --current_round "$CURRENT_ROUND"
echo "Contract admin initialized successfully, round set to $CURRENT_ROUND"

echo STEP 4: Setup neural governance
stellar contract invoke \
  --id $NEURAL_GOVERNANCE_ADDRESS \
  --source-account $STELLAR_SECRET_KEY \
  --rpc-url $STELLAR_RPC_URL \
  --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
  -- add_layer \
  --raw_neurons '[["Neuron1","1000000000000000000"],["Neuron2","1000000000000000000"]]' \
  --layer_aggregator "Sum"

stellar contract invoke \
  --id $NEURAL_GOVERNANCE_ADDRESS \
  --source-account $STELLAR_SECRET_KEY \
  --rpc-url $STELLAR_RPC_URL \
  --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
  -- add_layer \
  --raw_neurons '[["Neuron3","1000000000000000000"]]' \
  --layer_aggregator "Product"

echo "Neural governance set up successfully"

echo STEP 5: Update .env file
SED_IN_PLACE_OPTION="-i"

if [[ "$OSTYPE" == "darwin"* ]]; then
    SED_IN_PLACE_OPTION="-i ''"
fi

if grep -q "^NEURAL_GOVERNANCE_ADDRESS=" "$ENV_PATH"; then
    eval sed $SED_IN_PLACE_OPTION "s/^NEURAL_GOVERNANCE_ADDRESS=.*/NEURAL_GOVERNANCE_ADDRESS=$NEURAL_GOVERNANCE_ADDRESS/" "$ENV_PATH"
    echo "NEURAL_GOVERNANCE_ADDRESS has been updated."
else
    echo "NEURAL_GOVERNANCE_ADDRESS not found. Adding NEURAL_GOVERNANCE_ADDRESS to the .env file."
    echo "NEURAL_GOVERNANCE_ADDRESS=$NEURAL_GOVERNANCE_ADDRESS" >> "$ENV_PATH"
fi

echo DONE.