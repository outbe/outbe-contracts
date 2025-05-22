**This file documents the Nod contract interface and helper scripts**

# Nod

Nod smart contract built on the outbe-nft (CW721) standard.

This contract stores the following data in each NFT metadata's `extension`:

```json
{
  "nod_id": "string",
  "settlement_token": { "native": "string" },
  "symbolic_rate": "string",
  "vector_rate": "string",
  "nominal_minor_rate": "string",
  "issuance_minor_rate": "string",
  "symbolic_minor_load": "string",
  "vector_minor_rate": "string",
  "floor_minor_price": "string",
  "state": "Issued" | "Settled",
  "address": "string",
  "created_at": null | { "secs": number, "nanos": number }
}
```

## JSON example (Submit/mint)

Below is an example of an `ExecuteMsg::Submit` (mint) payload for Nod:

```json
{
  "submit": {
    "token_id": "nod-001",
    "owner": "cosmos1owneraddress",
    "extension": {
      "entity": {
        "nod_id": "nod-001",
        "settlement_token": { "native": "uatom" },
        "symbolic_rate": "1.234",
        "vector_rate": "0.567",
        "nominal_minor_rate": "1000",
        "issuance_minor_rate": "2000",
        "symbolic_minor_load": "3000",
        "vector_minor_rate": "4000",
        "floor_minor_price": "5000",
        "state": "Issued",
        "address": "cosmos1owneraddress"
      },
      "created_at": null
    }
  }
}
```

## Helper script: scripts/create_nod.ts

A TypeScript script to generate a Nod `Submit` message payload from:
1. A JSON file with the entity fields
2. Fake/random data for quick testing.

### Prerequisites

```sh
npm install --save-dev ts-node typescript @types/node yargs faker uuid \
  @cosmjs/proto-signing @cosmjs/cosmwasm-stargate
```

### Usage

Generate fake data:

```sh
npx ts-node scripts/create_nod.ts --fake --owner <owner-address> --token-id <token-id>
```

Read entity from a JSON file:

```sh
npx ts-node scripts/create_nod.ts --from-file path/to/entity.json --owner <owner-address> --token-id <token-id>
```

The script will print the complete `submit` message JSON to stdout.

### Broadcasting transaction

To sign and broadcast the created Nod on a Cosmos SDK network, install the Cosmos SDK client libraries:

```sh
npm install --save-dev @cosmjs/proto-signing @cosmjs/cosmwasm-stargate
```

Run the script with RPC options and your signer mnemonic:

```sh
npx ts-node scripts/create_nod.ts --fake --owner <owner-address> --token-id <token-id> \
  --rpc-url <rpc-endpoint> --contract <contract-address> \
  --mnemonic "<mnemonic phrase>" [--mnemonic-file <path>] [--prefix <bech32-prefix>] [--memo <memo>]
```

This will execute the `Submit` message on-chain, mint the Nod NFT, and print the transaction result (including `txHash`).