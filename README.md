# Outbe Smart Contracts

[![codecov](https://codecov.io/gh/outbe/outbe-contracts/graph/badge.svg?token=8OI56ZGYQQ)](https://codecov.io/gh/outbe/outbe-contracts)

This repository contains the source code for the Outbe Protocol smart contracts.
We have two types of contracts; dependency/test and main contracts.

Dependency/test contracts:

- **cw20**: CW20 is a standard for fungible tokens on the CosmWasm platform.

Main contracts:

- **tribute**: please see [docs](contracts/tribute/README.md) for more info.
- **vector**: please see [docs](contracts/vector/README.md) for more info.

## Requirements

Please check requirements at the official [CosmWasm book](https://book.cosmwasm.com/setting-up-env.html).

## How to build

To build the Wasm binary containing smart contracts, run:

```shell
cargo wasm
```

When the contract is built, to ensure it is a valid CosmWasm contract is to call cosmwasm-check on it:

```shell
cosmwasm-check ./target/wasm32-unknown-unknown/release/contract.wasm
```

To run unit tests:

```shell
cargo test
```

To run integration tests:

```shell
cargo integration-test
```

## Deploy

_TBD: in the near future CI / CD pipelines will be added to manually run smart contract deployments
via GitHub Actions UI._

### Build For Deployment

To deploy the smart contracts to a network, you need to build them and optimize.
Optimize is a process of shrinking the binary to fit into `store-core` transaction
which is done by the standard CosmWasm [optimizer](https://github.com/CosmWasm/optimizer).

_Just in case of any questions, please refer to the [official docs](https://cosmwasm.cosmos.network/wasmd/getting-started/cli#upload-code)._

To make an optimized build, run the following command:

```shell
docker run --rm \
  -v "$(pwd)":/code \
  -v ./docker_output:/target \
  ghcr.io/outbe/outbe-wasm-builder:latest optimize.sh .
```

It will produce `*.wasm` artifacts in the `./docker_output` directory.

### Ensure Deployment Wallet

Such as smart contracts deployment is an on-chain transaction you need to make sure that
you have a wallet with funds to be used for deployment.

To create a wallet:

```shell
outbe-noded keys add deployer
```

You will see a new wallet address created with the name "deployer". **Make sure to back up the seed.**

add some tokens to wallet on testnet

```
./outbe-noded tx bank send outbe140fehngcrxvhdt84x729p3f0qmkmea8nqxn3gl $deployer_address 10000000000outbe --chain-id localchain_90001-1 --from acc0
```

### Onchain Deployment

Now you can deploy the contracts on-chain. To do so run the following script: 

```shell
CHAIN_ID=localchain_90001-1
FEE_DENOM=outbe
RPC=http://localhost:26657

TX_HASH=$(outbe-noded tx wasm store $filename \
  -y --from deployer --broadcast-mode sync \
  --node $RPC --chain-id $CHAIN_ID --gas auto --gas-adjustment 1.3 --output json \
  | jq -r '.txhash')

sleep 7

CODE_ID=$(outbe-noded query tx --type=hash $TX_HASH --node $RPC --output json | \
  jq -r '.events[] | select(.type == "store_code") | .attributes[] | select(.key == "code_id") | .value')

echo $CODE_ID
```

At the end you will have `$CODE_ID`: unique identifier of the smart contract code deployed on-chain.
Now you can use this code to create a first instance of a smart contract:

```shell
CHAIN_ID=localchain_90001-1
FEE_DENOM=outbe
RPC=http://localhost:26657
INIT_PAYLOAD="<provide here InstantiateMsg JSON>"

TX_HASH=$(outbe-noded tx wasm instantiate \
  $CODE_ID "$INIT_PAYLOAD" \
  --label "<Your Smart Contract Label>" \
  --from deployer -y \
  --admin "<Your Deployer Wallet Address>" \
  --node $RPC --chain-id $CHAIN_ID --gas auto --gas-adjustment 1.3 --output json \
  | jq -r '.txhash')

sleep 7

# Query a created contract
# NB: we also need to filter by code_id because it may create several contracts under the hood
CONTRACT_ADDRESS=$(outbe-noded query tx --type=hash $TX_HASH --node $RPC --output json \
  | jq -r ".events[] | select(.type == \"instantiate\" and .attributes[].key == \"code_id\" and .attributes[].value == \"$CODE_ID\")  | .attributes[] | select(.key == \"_contract_address\") | .value")
```

At the end you will have `$CONTRACT_ADDRESS`: unique address of the smart contract instance deployed on-chain.

## Working with the protocol

Every smart contract has its own message set. You can find them in their README files.
In this section, we will show you how to interact with the protocol.

### Querying a smart contract

To query a smart contract, you can use the following command:

```shell
outbe-noded $NODE query wasm contract-state smart [CONTRACT_ADDRESS] [QUERY]
```

Query is a JSON object that you create based on the query message. Let's say you have this query message:

```rust
pub enum QueryMsg {
    Tokens {
        owner: Addr,
        start_after: Option<String>,
        limit: Option<u32>,
    },
}
```

You can create a query message like this:

```shell
outbe-noded $NODE query wasm contract-state smart [CONTRACT_ADDRESS] '{"tokens": {"owner": $address, "start_after": null, "limit": 10}}'
```

### Execute a smart contract

To execute a smart contract you can use the following command:

```shell
outbe-noded tx wasm execute [CONTRACT_ADDRESS] [EXECUTE_MESSAGE] --from [YOUR_ADDRESS] $TXFLAG
outbe-noded tx wasm execute [CONTRACT_ADDRESS] '{"minter": {"grant": {"address": $address}}}' --from $wallet $TXFLAG
```

Execute message is a JSON object that you create based on the execute message. Let's say you have this execute message:

```rust
pub enum ExecuteMsg {
    Mint {
        recipient: Addr,
    },
}
```

You can create an execute message like this:

```shell
outbe-noded tx wasm execute [CONTRACT_ADDRESS] '{"mint": {"recipient": $address}}' --from $wallet $TXFLAG
```

## Integration Tests

You can access to our updated flow integration tests from the `./integrations-tests/tests/multitest.rs` file.
