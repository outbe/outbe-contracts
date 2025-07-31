# Price Oracle Contract

A CosmWasm smart contract that provides decentralized price feeds for token pairs on the Outbe ecosystem. The contract allows authorized creators to update prices and maintains a comprehensive price history for each trading pair.

## Features

- **Token Pair Management**: Add and remove trading pairs dynamically
- **Price Updates**: Support for both simple price updates and comprehensive OHLC (Open, High, Low, Close) data
- **Price History**: Maintains complete price history with timestamps for each pair
- **Day Type Management**: Set and query current day type (Green/Red) for each pair
- **Time-based Queries**: Query historical prices within specific time ranges
- **Access Control**: Only authorized creators can update prices and manage pairs
- **Backward Compatibility**: Maintains support for legacy price update methods

## Installation

This contract is part of the Outbe contracts workspace. To build:

```bash
cargo build --release --target wasm32-unknown-unknown
```

## Instantiation

Initialize the contract with a creator address and initial price data:

```json
{
  "creator": "outbe1..." // Optional, defaults to sender
}
```

## Execute Messages

### Update Price
Enhanced price update with OHLC data support:

```json
{
  "update_price_v2": {
    "token1": { "native": "coen" },
    "token2": { "native": "wUSDC" },
    "price": "10.5",
    "open": "10.2",
    "high": "10.8",
    "low": "10.1",
    "close": "10.5"
  }
}
```

### Add Token Pair
Register a new trading pair:

```json
{
  "add_token_pair": {
    "token1": { "native": "coen" },
    "token2": { "native": "wUSDC" }
  }
}
```

### Remove Token Pair
Remove an existing trading pair and its price history:

```json
{
  "remove_token_pair": {
    "token1": { "native": "coen" },
    "token2": { "native": "wUSDC" }
  }
}
```

### Set Day Type
Set the current day type (Green/Red) for a specific token pair:

```json
{
  "set_day_type": {
    "token1": { "native": "coen" },
    "token2": { "native": "wUSDC" },
    "day_type": "Green"
  }
}
```

## Query Messages

### Get Price (Legacy)
Returns the legacy price format:

```json
{
  "get_price": {}
}
```

### Get Latest Price
Query the most recent price for a specific pair:

```json
{
  "get_latest_price": {
    "token1": { "native": "coen" },
    "token2": { "native": "wUSDC" }
  }
}
```

### Get Price History
Query historical prices within a time range:

```json
{
  "get_price_history": {
    "token1": { "native": "coen" },
    "token2": { "native": "wUSDC" },
    "start_time": 1700000000,
    "end_time": 1700086400
  }
}
```

### Get All Pairs
List all registered trading pairs:

```json
{
  "get_all_pairs": {}
}
```

### Get Creator Ownership
Query the current contract owner:

```json
{
  "get_creator_ownership": {}
}
```

### Get Day Type
Query the current day type for a specific token pair:

```json
{
  "get_day_type": {
    "token1": { "native": "coen" },
    "token2": { "native": "wUSDC" }
  }
}
```

## Data Types

### TokenPairPrice (Legacy)
```rust
{
  "token1": Denom,
  "token2": Denom,
  "day_type": "Green" | "Red",
  "price": Decimal
}
```

### PriceData
```rust
{
  "price": Decimal,
  "timestamp": Timestamp,
  "open": Option<Decimal>,
  "high": Option<Decimal>,
  "low": Option<Decimal>,
  "close": Option<Decimal>
}
```

### TokenPair
```rust
{
  "token1": Denom,
  "token2": Denom
}
```

## Security

- Only the contract creator can update prices and manage token pairs
- Token pairs must consist of different tokens
- All price updates are timestamped with block time
- Price history is immutable once written

## Events

The contract emits the following events:

- `price-oracle::instantiate` - Contract initialization
- `price-oracle::price_updated` - Legacy price update
- `price-oracle::price_updated_v2` - V2 price update with OHLC data
- `price-oracle::pair_added` - New trading pair registered
- `price-oracle::pair_removed` - Trading pair removed
- `price-oracle::day_type_set` - Day type updated for a pair