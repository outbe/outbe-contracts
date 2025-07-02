# Tribute

Tribute is a non-transferable non-fungible cryptographically secure and auditable token
that represents a proof of consumption made by a user and 
serves as inputs to the network’s Proof of Consumption logic 
and underpin token issuance and reward participants. 

Tribute creation is a responsibility of a separate actor an "Agent" that
signs each record, and it's pushed to the network.

_NB: at the moment anyone could create Tributes for testing purpose i.e. no mint restrictions_

Tribute is implemented in the way of smart contract, and below is described logic how to deal with it.

## Tribute Creation

Creating of the Tribute is a "mint" operation. 
For minting a Tribute the following data is required:
- "token_id" the Tribute identifier, should be unique.
- "owner" a user address that this consumption belongs to.
- "data" the Tribute entity i.e. payload info.
- "signature" is a cryptographical signature of the entity hash.
- "public_key" is a public key to verify the signature.
- "token_uri" is an optional URL for compatibility with CW721.

### Signing Tribute Data

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

Please see an example of the signature creation [in Rust](./src/contract.rs:335).

Please see an example of the signature creation [in TypeScript](tribute-ts-scripts/README.md).

## Consumption Unit deployment info

Devnet deployment address: `outbe1s4683e9zlq2pd2en2gnxrqzer0jvq3cj86qgx6r69h4n3j7vcsfsa2wwsh`

Queries: 

```shell
CONTRACT_ADDRESS=outbe1s4683e9zlq2pd2en2gnxrqzer0jvq3cj86qgx6r69h4n3j7vcsfsa2wwsh
outbe-noded query wasm contract-state smart $CONTRACT_ADDRESS '{"contract_info": {}}' --node $RPC
outbe-noded query wasm contract-state smart $CONTRACT_ADDRESS '{"all_tokens": {}}' --node $RPC

DATA=$(echo '<mint payload>' | jq )

outbe-noded tx wasm execute $CONTRACT_ADDRESS "$DATA" \
  --node $RPC --from ci --chain-id $CHAIN_ID \
  --gas auto --gas-adjustment 1.5 --gas-prices 0.025$FEE_DENOM -y

outbe-noded query wasm contract-state smart $CONTRACT_ADDRESS '{"nft_info": {"token_id" : "1"}}' --node $RPC

```
