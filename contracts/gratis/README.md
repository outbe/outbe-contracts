# Gratis Smart Contract

A CosmWasm soulbound CW20 token with burn-to-ticket functionality.

## Overview

Gratis is a soulbound CW20 token that implements a unique burn mechanism generating permanent tickets as proof of burned tokens. The contract uses Blake3 hashing for secure ticket generation and is completely non-transferable.

## Features

- **Soulbound Token**: Fully non-transferable token (no Transfer, TransferFrom, Send)
- **Burn-to-Ticket System**: Burning tokens generates permanent tickets as cryptographic proof
- **One Burn Per Block**: Users can only burn once per block height to prevent spam
- **Blake3 Hashing**: Secure and efficient ticket generation
- **6 Decimal Places**: Standard precision for token amounts

## Token Details

- **Name**: "Gratis"
- **Symbol**: "GRATIS"
- **Decimals**: 6
- **Initial Supply**: 0 (minted as needed)

## Technical Specifications

### Ticket Generation

Tickets are generated using the Blake3 hash algorithm with the formula:
```
ticket = blake3_hash(burner_address + "," + amount + "," + block_height)
```

This ensures:
- **Uniqueness**: Each ticket is unique based on address, amount, and block height
- **Verifiability**: Tickets can be independently verified
- **Permanence**: Tickets are stored permanently in contract state

### State Storage

- `TICKETS`: Map<String, bool> - Stores generated ticket hashes
- `USER_BURNS_PER_BLOCK`: Map<(Addr, u64), bool> - Tracks burns per user per block

## Installation

### Prerequisites
- Rust 1.70+
- CosmWasm 2.0.3
- cargo-generate

### Build
```bash
cd gratis
cargo build --release --target wasm32-unknown-unknown
```

### Testing
```bash
cargo test
```

## Usage

### Deployment

```json
{
  "mint": {
    "minter": "outbe1...",
    "cap": null
  }
}
```

### Execute Messages

#### Burn Tokens
Burns tokens and generates a permanent ticket.

```json
{
  "burn": {
    "amount": "1000000"
  }
}
```

**Response Attributes**:
- `action`: "burn"
- `from`: burner address
- `amount`: burned amount
- `ticket`: generated ticket hash
- `block_height`: block height when burned


#### Mint Tokens (Minter Only)
```json
{
  "mint": {
    "recipient": "outbe1...",
    "amount": "1000000"
  }
}
```

#### Update Minter (Current Minter Only)
```json
{
  "update_minter": {
    "new_minter": "cosmos1..."
  }
}
```

### Query Messages

#### Check Ticket Existence
Verifies if a ticket exists in the contract state.

```json
{
  "check_ticket": {
    "ticket": "a1b2c3d4e5f6..."
  }
}
```

**Response**:
```json
{
  "exists": true
}
```

#### Get Balance
```json
{
  "balance": {
    "address": "outbe1..."
  }
}
```

#### Get Token Info
```json
{
  "token_info": {}
}
```

**Response**:
```json
{
  "name": "Gratis",
  "symbol": "GRATIS",
  "decimals": 6,
  "total_supply": "1000000"
}
```

#### Get Minter Info
```json
{
  "minter": {}
}
```

#### Get All Accounts
```json
{
  "all_accounts": {
    "start_after": "outbe1...",
    "limit": 30
  }
}
```

## Security Features

1. **Soulbound Implementation**: Completely prevents all token transfers
2. **One Burn Per Block**: Prevents spam attacks and ensures fair distribution
3. **Blake3 Hashing**: Cryptographically secure ticket generation
4. **Minter Controls**: Only authorized minter can create new tokens

## Error Handling

- `AlreadyBurnedInBlock`: User attempted multiple burns in same block
- `Unauthorized`: Insufficient permissions for minting operations
- `InvalidZeroAmount`: Attempted operation with zero amount
- `InsufficientFunds`: Insufficient token balance for operation

## Use Cases

1. **Proof of Burn**: Generate verifiable proof that tokens were destroyed
2. **Governance**: Use burn tickets as voting power or participation proof
3. **Rewards**: Burn tokens to claim rewards or benefits
4. **Deflationary Mechanics**: Reduce token supply with permanent proof
5. **Achievement Systems**: Permanent proof of participation or milestones

## Gas Optimization

- Efficient storage patterns using cw-storage-plus
- Minimal state updates during operations
- Optimized error handling
- Batch validation where possible

## Development

### Adding New Features
1. Update `msg.rs` with new message types
2. Implement handlers in `contract.rs`
3. Add corresponding tests
4. Update documentation

### Testing Guidelines
- Test all burn scenarios including edge cases
- Verify ticket generation and uniqueness
- Validate security restrictions
- Test error conditions

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add comprehensive tests
4. Ensure all tests pass
5. Submit a pull request

## License

This project is licensed under the MIT License.

## Support

For questions and support, please open an issue in the repository.
