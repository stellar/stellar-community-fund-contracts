#!/bin/bash
ENV_PATH=".env"
source $ENV_PATH

SUBMISSIONS_FILE="./data/neurons_output.json"

jq -c '.[]' "$SUBMISSIONS_FILE" | while read -r row; do
    name=$(echo "$row" | jq -r '.name')
    category=$(echo "$row" | jq -r '.category')

echo "Uploading neuron1"
stellar contract invoke \
    --id $NEURAL_GOVERNANCE_ADDRESS \
    --source-account $STELLAR_SECRET_KEY \
    --rpc-url $STELLAR_RPC_URL \
    --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
    -- set_neuron_result \
    --layer_id="0" \
    --neuron_id="0" \

echo "Uploading neuron2"
stellar contract invoke \
    --id $NEURAL_GOVERNANCE_ADDRESS \
    --source-account $STELLAR_SECRET_KEY \
    --rpc-url $STELLAR_RPC_URL \
    --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
    -- set_neuron_result \
    --layer_id="0" \
    --neuron_id="1" \

echo "Uploading neuron3"
stellar contract invoke \
    --id $NEURAL_GOVERNANCE_ADDRESS \
    --source-account $STELLAR_SECRET_KEY \
    --rpc-url $STELLAR_RPC_URL \
    --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
    -- set_neuron_result \
    --layer_id="1" \
    --neuron_id="0" \
