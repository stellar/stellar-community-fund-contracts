#!/bin/bash
ENV_PATH=".env"
source $ENV_PATH

ADDRESS="user3"

stellar contract invoke \
  --id $SCF_TOKEN_ADDRESS \
  --source-account $STELLAR_SECRET_KEY \
  --rpc-url $STELLAR_RPC_URL \
  --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
  -- update_balance \
  --address=$ADDRESS
