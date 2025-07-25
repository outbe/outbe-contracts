#!/usr/bin/env sh
set -e

contract_address=$1
message=$2
extra_args=$3

TX_HASH=$(outbe-noded tx wasm execute $contract_address "$message" \
  --node $RPC --from ci --keyring-backend test --chain-id $CHAIN_ID \
  --gas auto --gas-adjustment 1.5 --gas-prices 0.025$FEE_DENOM -y --output json $extra_args \
  | jq -r '.txhash')

sleep 7

echo $TX_HASH
