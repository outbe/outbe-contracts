#!/usr/bin/env sh
set -e

filename=$1

TX_HASH=$(outbe-noded tx wasm store $filename \
  -y --from ci --keyring-backend test --broadcast-mode sync \
  --node $RPC --chain-id $CHAIN_ID --gas-prices 0.25$FEE_DENOM --gas auto --gas-adjustment 1.3 --output json \
  | jq -r '.txhash')

sleep 7

CODE_ID=$(outbe-noded query tx --type=hash $TX_HASH --node $RPC --output json | \
  jq -r '.events[] | select(.type == "store_code") | .attributes[] | select(.key == "code_id") | .value')

echo $CODE_ID
