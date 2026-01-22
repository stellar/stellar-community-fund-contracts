#!/bin/bash
ENV_PATH=".env"
source $ENV_PATH

VOTES_FILE="./data/votes.json"

for row in $(jq -r '. | to_entries[] | @base64' "$VOTES_FILE"); do
    _decode() {
     echo ${row} | base64 --decode
    }
    name=$(_decode | jq -r '.key')
    votes=$(_decode | jq -c '.value')

    echo "Uploading votes for submission $name "
    stellar contract invoke \
        --id $NEURAL_GOVERNANCE_ADDRESS \
        --source-account $STELLAR_SECRET_KEY \
        --rpc-url $STELLAR_RPC_URL \
        --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
        -- set_votes_for_submission \
        --submission_id=$name \
        --votes=$votes
done