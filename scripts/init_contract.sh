#!/usr/bin/env sh
set -e

label=$1
code_id=$2
payload=$3

binary=${BINARY:-outbe-noded}

RESPONSE=$($binary tx wasm instantiate \
  $code_id "$payload" \
  --label "$label" \
  --from ci --keyring-backend test -y --admin $($binary keys show --keyring-backend test ci -a) \
  --node $RPC --chain-id $CHAIN_ID --gas-prices 0.25$FEE_DENOM --gas auto --gas-adjustment 1.3 --output json)
TX_HASH=$(echo "$RESPONSE" | jq -r '.txhash')

sleep 7

json_filter=".events[] | select(.type == \"instantiate\" and .attributes[].key == \"code_id\" and .attributes[].value == \"$code_id\")  | .attributes[] | select(.key == \"_contract_address\") | .value"
# it is is located in the logs for SEI
if [ "$binary" != "outbe-noded" ]; then
  json_filter=".logs[] | $json_filter"
fi

# Query a created contract
# NB: we also need to filter by code_id because it may create several contracts under the hood
RESPONSE=$($binary query tx --type=hash $TX_HASH --node $RPC --output json)
CONTRACT_ADDRESS=$(echo "$RESPONSE"| jq -r "$json_filter")

echo $CONTRACT_ADDRESS
