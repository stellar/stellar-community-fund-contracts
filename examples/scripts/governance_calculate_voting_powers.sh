#!/bin/bash
ENV_PATH=".env"
source $ENV_PATH

echo "Calculating voting powers"
stellar contract invoke \
  --id $NEURAL_GOVERNANCE_ADDRESS \
  --source-account $STELLAR_SECRET_KEY \
  --rpc-url $STELLAR_RPC_URL \
  --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
  -- calculate_voting_powers