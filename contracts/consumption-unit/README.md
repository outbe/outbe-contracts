# Consumption Unit

Consumption Unit is a non-transferable non-fungible cryptographically secure and auditable token
that represents a proof of consumption made by a user and 
serves as inputs to the network’s Proof of Consumption logic 
and underpin token issuance and reward participants. 

Consumption Units creation is a responsibility of a separate actor "Consumption Unit Agent" that
signs each consumption unit and pushes to the network.

_NB: at the moment anyone could create consumption units for testing purpose i.e. no mint restrictions_

Consumption unit is implemented in a way of smart contract and below is described logic how to deal with it.

## Consumption Unit Creation

Creating of the consumption unit is a "mint" operation. 
For minting a consumption unit the following data is required:
- "token_id" the consumption unit identifier, should be unique.
- "owner" a user address that this consumption belongs to.
- "entity" the consumption unit entity i.e. payload info.
- "signature" is a cryptographical signature of the entity hash.
- "public_key" is a public key to verify the signature.

### Signing Consumption Unit Data

Signing raw data is important to make sure that the given Consumption Unit is authentic. For assuring that
the [ECDSA secp256k1](https://cosmwasm.cosmos.network/core/standard-library/cryptography/k256) elliptic curve cryptography is used.
It's a Koblitz curve widely used in the blockchain space (e.g., Bitcoin and Ethereum).

To create a valid signature, you need to perform the following steps:

- serialize the given entity as a binary json in [json binary](https://github.com/CosmWasm/serde-json-wasm)
- produce a hash of the given bytes using `sha256` hash function
- sign the given hash using secp256k1 cryptography
- provide the given signature and public key in hex format within the message

Note:  
The signature and public key are in "Cosmos" format:  
signature: Serialized "compact" signature (64 bytes).  
public key: Serialized according to SEC 2 (33 or 65 bytes).  

_This implementation accepts both high-S and low-S signatures.
Some applications, including Ethereum transactions, consider high-S signatures invalid to avoid malleability.
If that's the case for your protocol, the signature needs to be tested for low-S in addition to this verification._

Please see an example of the signature creation [in Rust](./src/contract.rs:295).

Please see an example of the signature creation [in TypeScript](./sign-cu-ts-demo/README.md).

## Consumption Unit deployment info

Devnet deployment address: `gem15m5fe2pfxq6796rf2z7gma8a0n2s0f0dxasmrzark3q26tltgsyschkllh`

Queries: 

```shell
CONTRACT_ADDRESS=gem15m5fe2pfxq6796rf2z7gma8a0n2s0f0dxasmrzark3q26tltgsyschkllh
gemchaind query wasm contract-state smart $CONTRACT_ADDRESS '{"contract_info": {}}' --node $RPC
gemchaind query wasm contract-state smart $CONTRACT_ADDRESS '{"all_tokens": {}}' --node $RPC

DATA=$(echo '{"mint":{"token_id":"1","owner":"cosmwasm1j2mmggve9m6fpuahtzvwcrj3rud9cqjz9qva39cekgpk9vprae8s4haddx","extension":{"entity":{"token_id":"1","owner":"cosmwasm1j2mmggve9m6fpuahtzvwcrj3rud9cqjz9qva39cekgpk9vprae8s4haddx","consumption_value":"100","nominal_quantity":"100","nominal_currency":"usd","commitment_tier":1,"hashes":["hash1"]},"digest":"872be89dd82bcc6cf949d718f9274a624c927cfc91905f2bbb72fa44c9ea876d"}}}' | jq )

gemchaind tx wasm execute $CONTRACT_ADDRESS "$DATA" \
  --node $RPC --from ci --chain-id $CHAIN_ID \
  --gas auto --gas-adjustment 1.5 --gas-prices 0.025$FEE_DENOM -y

gemchaind query wasm contract-state smart $CONTRACT_ADDRESS '{"nft_info": {"token_id" : "1"}}' --node $RPC

```
