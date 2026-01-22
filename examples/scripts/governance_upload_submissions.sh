#!/bin/bash
ENV_PATH=".env"
source $ENV_PATH

SUBMISSIONS_FILE="./data/submissions.json"

SUBMISSIONS_ARRAY=$(sed 's/,[[:space:]]*}/}/g' "$SUBMISSIONS_FILE" | jq -s -c '
  flatten | map(select(. != null) | [.name, .category])
')

echo "Uploading submissions: $SUBMISSIONS_ARRAY"

stellar contract invoke \
    --id $NEURAL_GOVERNANCE_ADDRESS \
    --source-account $STELLAR_SECRET_KEY \
    --rpc-url $STELLAR_RPC_URL \
    --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
    -- set_submissions \
    --new_submissions_raw=$SUBMISSIONS_ARRAY \