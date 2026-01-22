#!/bin/bash
ENV_PATH=".env"
source $ENV_PATH

SUBMISSIONS_FILE="./data/submissions.json"

jq -c '.[]' "$SUBMISSIONS_FILE" | while read -r row; do
    name=$(echo "$row" | jq -r '.name')
    category=$(echo "$row" | jq -r '.category')

echo "Uploading submissions"
stellar contract invoke \
        --id $NEURAL_GOVERNANCE_ADDRESS \
        --source-account $STELLAR_SECRET_KEY \
        --rpc-url $STELLAR_RPC_URL \
        --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
        -- set_submissions \
        --new_submissions_raw=$name \