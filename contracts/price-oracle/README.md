# Price Oracle Contract

A CosmWasm smart contract that provides decentralized price feeds for token pairs on the Outbe ecosystem. The contract supports advanced price analytics including VWAP (Volume Weighted Average Price) calculations and maintains comprehensive price history for each trading pair.

## Features

- **Token Pair Management**: Add and remove trading pairs dynamically
- **Price Updates**: Support for both simple price updates and comprehensive OHLC (Open, High, Low, Close) data with volume
- **VWAP Calculation**: Automatic Volume Weighted Average Price calculation with configurable time windows
- **VWAP History**: Maintains complete VWAP history for historical analysis
- **Price History**: Maintains complete price history with timestamps for each pair
- **Day Type Management**: Set and query current day type (Green/Red) for each pair
- **Time-based Queries**: Query historical prices and VWAP within specific time ranges
- **Access Control**: Only authorized creators can update prices and manage pairs
- **Backward Compatibility**: Maintains support for legacy price update methods

## Installation

This contract is part of the Outbe contracts workspace. To build:

```bash
cargo build --release --target wasm32-unknown-unknown
```

## Instantiation

Initialize the contract with a creator address and VWAP configuration:

```json
{
  "creator": "outbe1...", // Optional, defaults to sender
  "vwap_window_seconds": 300 // Optional, defaults to 300 seconds (5 minutes)
}
```

## Execute Messages

### Update Price
Enhanced price update with OHLC data and volume support:

```json
{
  "update_price": {
    "token1": { "native": "coen" },
    "token2": { "native": "wUSDC" },
    "price": "10.5",
    "open": "10.2",
    "high": "10.8",
    "low": "10.1",
    "close": "10.5",
    "volume": "1000000" // Optional, required for VWAP calculation
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
Remove an existing trading pair and its price/VWAP history:

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

### Update VWAP Window
Configure the time window for VWAP calculations:

```json
{
  "update_vwap_window": {
    "window_seconds": 600 // 10 minutes
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

### Get VWAP
Query the current Volume Weighted Average Price:

```json
{
  "get_vwap": {
    "token1": { "native": "coen" },
    "token2": { "native": "wUSDC" }
  }
}
```

Returns:
```json
{
  "vwap": "10.45",
  "total_volume": "5000000",
  "window_seconds": 300,
  "timestamp": 1700086400
}
```

### Get VWAP History
Query historical VWAP values within a time range:

```json
{
  "get_vwap_history": {
    "token1": { "native": "coen" },
    "token2": { "native": "wUSDC" },
    "start_time": 1700000000,
    "end_time": 1700086400
  }
}
```

### Get VWAP Config
Query the current VWAP window configuration:

```json
{
  "get_vwap_config": {}
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
  "close": Option<Decimal>,
  "volume": Option<Uint128>
}
```

### VwapData
```rust
{
  "vwap": Decimal,
  "total_volume": Uint128,
  "window_seconds": u64,
  "timestamp": Timestamp
}
```

### VwapConfig
```rust
{
  "window_seconds": u64
}
```

### TokenPair
```rust
{
  "token1": Denom,
  "token2": Denom
}
```

## VWAP Calculation

The Volume Weighted Average Price (VWAP) is calculated using the formula:

```
VWAP = Σ(Price × Volume) / Σ(Volume)
```

Where the summation is performed over all price updates within the configured time window.

### Key Features:
- **Automatic Calculation**: VWAP is automatically calculated when prices are updated with volume data
- **Configurable Window**: Default 5-minute window, adjustable via `update_vwap_window`
- **Historical Tracking**: All VWAP calculations are stored for historical analysis
- **Time-based Filtering**: Only includes price data within the specified window

### Use Cases:
- **Fair Value Assessment**: Determine if current price is above or below average traded price
- **Trading Benchmarks**: Use as a reference price for large orders
- **Market Analysis**: Analyze price trends weighted by actual trading volume
- **Execution Quality**: Measure trading performance against VWAP benchmark

## Security

- Only the contract creator can update prices and manage token pairs
- Token pairs must consist of different tokens
- All price updates are timestamped with block time
- Price and VWAP history is immutable once written
- Time ranges must be valid (start_time < end_time)

## Events

The contract emits the following events:

- `price-oracle::instantiate` - Contract initialization
- `price-oracle::price_updated` - Price update with OHLC and volume data
- `price-oracle::pair_added` - New trading pair registered
- `price-oracle::pair_removed` - Trading pair removed
- `price-oracle::day_type_set` - Day type updated for a pair
- `price-oracle::update_vwap_window` - VWAP window configuration updated

## Example Usage

### Setting up a new trading pair with VWAP tracking:

1. Add the token pair:
```json
{
  "add_token_pair": {
    "token1": { "native": "ATOM" },
    "token2": { "native": "USDC" }
  }
}
```

2. Update price with volume for VWAP calculation:
```json
{
  "update_price": {
    "token1": { "native": "ATOM" },
    "token2": { "native": "USDC" },
    "price": "9.75",
    "volume": "150000"
  }
}
```

3. Query current VWAP:
```json
{
  "get_vwap": {
    "token1": { "native": "ATOM" },
    "token2": { "native": "USDC" }
  }
}
```

4. Query VWAP history for analysis:
```json
{
  "get_vwap_history": {
    "token1": { "native": "ATOM" },
    "token2": { "native": "USDC" },
    "start_time": 1700000000,
    "end_time": 1700086400
  }
}
```