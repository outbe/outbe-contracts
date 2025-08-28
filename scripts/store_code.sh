#!/usr/bin/env sh
set -e

filename=$1

binary=${BINARY:-outbe-noded}

json_filter='.events[] | select(.type == "store_code") | .attributes[] | select(.key == "code_id") | .value'
# code is is located in the logs for SEI
if [ "$binary" != "outbe-noded" ]; then
  json_filter=".logs[] | $json_filter"
fi

TX_HASH=$($binary tx wasm store $filename \
  -y --from ci --keyring-backend test --broadcast-mode sync \
  --node $RPC --chain-id $CHAIN_ID --gas-prices 0.25$FEE_DENOM --gas auto --gas-adjustment 1.3 --output json \
  | jq -r '.txhash')

sleep 7

CODE_ID=$($binary query tx --type=hash $TX_HASH --node $RPC --output json | jq -r "$json_filter")

echo $CODE_ID
