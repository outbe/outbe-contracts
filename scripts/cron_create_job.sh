#!/usr/bin/env sh
set -e

get_tomorrow_date() {
    date -u -v+1d '+%Y-%m-%d'
}

job_name=$1
message=$2
time=$3

binary=${BINARY:-outbe-chaind}

RESPONSE=$($binary tx cron create-job $job_name \
  --node $RPC --from ci --keyring-backend test --chain-id $CHAIN_ID \
  --gas auto --gas-adjustment 1.5 --gas-prices 0.025$FEE_DENOM -y --output json \
  --start-time "$(get_tomorrow_date)T$time" \
  --interval-seconds 86400 \
  --message "$message")

TX_HASH=$(echo "$RESPONSE" | jq -r '.txhash')
sleep 7

echo $TX_HASH
