# Consumption Unit

Devnet deployment address: `gem15m5fe2pfxq6796rf2z7gma8a0n2s0f0dxasmrzark3q26tltgsyschkllh`

Queries: 

```shell
CONTRACT_ADDRESS=gem15m5fe2pfxq6796rf2z7gma8a0n2s0f0dxasmrzark3q26tltgsyschkllh
gemchaind query wasm contract-state smart $CONTRACT_ADDRESS '{"contract_info": {}}' --node $RPC
gemchaind query wasm contract-state smart $CONTRACT_ADDRESS '{"all_tokens": {}}' --node $RPC

DATA=$(echo '{ "mint": { "token_id": "1", "owner": "gem1w893c3035l2400mre9cjfpnvqxd50wr977lzx4", "extension": { "consumption_value" : "10", "nominal_quantity" : "10",  "nominal_currency" : "USD",  "commitment_tier" : 1, "hashes": ["hash1"] } } }' | jq )

gemchaind tx wasm execute $CONTRACT_ADDRESS "$DATA" \
  --node $RPC --from ci --chain-id $CHAIN_ID \
  --gas auto --gas-adjustment 1.5 --gas-prices 0.025$FEE_DENOM -y

gemchaind query wasm contract-state smart $CONTRACT_ADDRESS '{"nft_info": {"token_id" : "1"}}' --node $RPC

```
