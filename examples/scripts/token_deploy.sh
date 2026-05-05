#!/bin/bash
# Script builds, deploys and initializes scf_token contract

ENV_PATH=".env"
source $ENV_PATH

echo STEP 1: Build all contracts
pushd "../contracts"
  stellar contract build 
popd

echo STEP 2: Deploy and initialize scf_token contract
SCF_TOKEN_ADDRESS="$(stellar contract deploy \
  --network $STELLAR_NETWORK \
  --wasm ../contracts/target/wasm32v1-none/release/scf_token.wasm \
  --rpc-url $STELLAR_RPC_URL \
  --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
  --source-account $STELLAR_SECRET_KEY \
  -- \
  --admin=$STELLAR_PUBLIC_KEY \
  --governance_address "$NEURAL_GOVERNANCE_ADDRESS")"

echo "SCF_TOKEN_ADDRESS: $SCF_TOKEN_ADDRESS"
echo "Contract admin initialized successfully, governance_address set to $NEURAL_GOVERNANCE_ADDRESS"

echo STEP 3: Update .env file
SED_IN_PLACE_OPTION="-i"

if [[ "$OSTYPE" == "darwin"* ]]; then
    SED_IN_PLACE_OPTION="-i ''"
fi

if grep -q "^SCF_TOKEN_ADDRESS=" "$ENV_PATH"; then
    eval sed $SED_IN_PLACE_OPTION "s/^SCF_TOKEN_ADDRESS=.*/SCF_TOKEN_ADDRESS=$SCF_TOKEN_ADDRESS/" "$ENV_PATH"
    echo "SCF_TOKEN_ADDRESS has been updated."
else
    echo "SCF_TOKEN_ADDRESS not found. Adding SCF_TOKEN_ADDRESS to the .env file."
    echo "SCF_TOKEN_ADDRESS=$SCF_TOKEN_ADDRESS" >> "$ENV_PATH"
fi

echo DONE.