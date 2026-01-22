#!/bin/bash
ENV_PATH=".env"
source $ENV_PATH

NEURONS_FILE="./data/neurons_output.json"

NEURON1=$(jq -c '.Neuron1' "$NEURONS_FILE")
NEURON2=$(jq -c '.Neuron2' "$NEURONS_FILE")
NEURON3=$(jq -c '.Neuron3' "$NEURONS_FILE")

echo "Uploading neuron1 data"
echo "$NEURON1"

stellar contract invoke \
    --id $NEURAL_GOVERNANCE_ADDRESS \
    --source-account $STELLAR_SECRET_KEY \
    --rpc-url $STELLAR_RPC_URL \
    --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
    -- set_neuron_result \
    --layer_id '{"string":"0"}' \
    --neuron_id '{"string":"0"}' \
    --result=$NEURON1

echo "Uploading neuron2 data"
echo "$NEURON2"

stellar contract invoke \
    --id $NEURAL_GOVERNANCE_ADDRESS \
    --source-account $STELLAR_SECRET_KEY \
    --rpc-url $STELLAR_RPC_URL \
    --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
    -- set_neuron_result \
    --layer_id '{"string":"0"}' \
    --neuron_id '{"string":"1"}' \
    --result=$NEURON2

echo "Uploading neuron3 data"
echo "$NEURON3"

stellar contract invoke \
    --id $NEURAL_GOVERNANCE_ADDRESS \
    --source-account $STELLAR_SECRET_KEY \
    --rpc-url $STELLAR_RPC_URL \
    --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
    -- set_neuron_result \
    --layer_id '{"string":"1"}' \
    --neuron_id '{"string":"0"}' \
    --result=$NEURON3