#!/bin/bash
ENV_PATH=".env"
source $ENV_PATH

echo "Setting round to $CURRENT_ROUND"
stellar contract invoke \
  --id $NEURAL_GOVERNANCE_ADDRESS \
  --source-account $STELLAR_SECRET_KEY \
  --rpc-url $STELLAR_RPC_URL \
  --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
  -- set_current_round \
  --round=$CURRENT_ROUND

ROUND=$(stellar contract invoke \
  --id $NEURAL_GOVERNANCE_ADDRESS \
  --source-account $STELLAR_SECRET_KEY \
  --rpc-url $STELLAR_RPC_URL \
  --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
  -- get_current_round)

echo "Round set to $ROUND"