#!/usr/bin/env sh
set -e

contract_address=$1
code_id=$2
payload=$3

binary=${BINARY:-outbe-noded}
payload=${payload:-'{ "migrate": {} }'}

$binary tx wasm migrate $contract_address \
  $code_id "$payload" \
  --from ci --keyring-backend test -y \
  --node $RPC --chain-id $CHAIN_ID --gas-prices 0.25$FEE_DENOM --gas auto --gas-adjustment 1.3 --output json
