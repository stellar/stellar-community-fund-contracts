#!/bin/bash
ENV_PATH=".env"
source $ENV_PATH

SUBMISSIONS_FILE="./data/submissions.json"

jq -r '.[].name' "$SUBMISSIONS_FILE" | while read -r submission_name; do
  RESULT=$(stellar contract invoke \
    --id $NEURAL_GOVERNANCE_ADDRESS \
    --source-account $STELLAR_SECRET_KEY \
    --rpc-url $STELLAR_RPC_URL \
    --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
    -- tally_submission \
    --submission_id=$submission_name)
  echo "$submission_name $RESULT"
done
