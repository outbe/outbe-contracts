# Promis Smart Contract

A CosmWasm soulbound CW20 token with burn-to-ticket functionality and 1:1 conversion to Gratis tokens.

## Overview

Promis is an advanced soulbound CW20 token that extends the burn-to-ticket functionality with the ability to convert tokens to Gratis at a 1:1 ratio. Like Gratis, it uses Blake3 hashing for secure ticket generation and is completely non-transferable.

## Features

- **Soulbound Token**: Fully non-transferable token (no Transfer, TransferFrom, Send)
- **Burn-to-Ticket System**: Burning tokens generates permanent tickets as cryptographic proof
- **Gratis Conversion**: Convert Promis tokens to Gratis tokens at 1:1 ratio
- **One Burn Per Block**: Users can only burn once per block height to prevent spam
- **Blake3 Hashing**: Secure and efficient ticket generation
- **Cross-contract Integration**: Automated minting of Gratis tokens during conversion
- **6 Decimal Places**: Standard precision for token amounts

## Token Details

- **Name**: "Promis"
- **Symbol**: "PROMIS"
- **Decimals**: 6
- **Initial Supply**: 0 (minted as needed)

## Technical Specifications

### Ticket Generation

Tickets are generated using the Blake3 hash algorithm with the formula:
```
ticket = blake3_hash(burner_address + "," + amount + "," + block_height)
```

### Gratis Conversion

The conversion process:
1. Burns Promis tokens from user's balance
2. Reduces total supply of Promis
3. Sends cross-contract message to Gratis contract
4. Mints equivalent amount of Gratis tokens to user

### State Storage

- `TICKETS`: Map<String, bool> - Stores generated ticket hashes
- `USER_BURNS_PER_BLOCK`: Map<(Addr, u64), bool> - Tracks burns per user per block
- `GRATIS_CONTRACT`: Item<Addr> - Address of Gratis contract for conversion

## Installation

### Prerequisites
- Rust 1.70+
- CosmWasm 2.0.3
- cargo-generate

### Build
```bash
cd promis
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
  },
  "gratis_contract": "outbe1..."
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

#### Convert to Gratis
Converts Promis tokens to Gratis tokens at 1:1 ratio.

```json
{
  "convert_to_gratis": {
    "amount": "1000000"
  }
}
```

**Response Attributes**:
- `action`: "convert_to_gratis"
- `from`: converter address
- `amount`: converted amount
- `gratis_contract`: Gratis contract address

**Process**:
1. Burns specified amount of Promis tokens
2. Sends mint message to Gratis contract
3. User receives equivalent Gratis tokens


#### Mint Tokens (Minter Only)
```json
{
  "mint": {
    "recipient": "outbe1...",
    "amount": "1000000"
  }
}
```

#### Burn From (With Allowance)
```json
{
  "burn_from": {
    "owner": "outbe1...",
    "amount": "1000000"
  }
}
```

#### Update Minter (Current Minter Only)
```json
{
  "update_minter": {
    "new_minter": "outbe1..."
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
  "name": "Promis",
  "symbol": "PROMIS",
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
3. **Address Validation**: Strict validation of Gratis contract address
4. **Blake3 Hashing**: Cryptographically secure ticket generation
5. **Cross-contract Security**: Validated addresses for inter-contract communication
6. **Minter Controls**: Only authorized minter can create new tokens

## Error Handling

- `AlreadyBurnedInBlock`: User attempted multiple burns in same block
- `Unauthorized`: Insufficient permissions for minting operations
- `InvalidZeroAmount`: Attempted operation with zero amount
- `InsufficientFunds`: Insufficient token balance for operation
- `InsufficientPromisBalance`: Insufficient Promis tokens for conversion

## Use Cases

1. **Proof of Burn**: Generate verifiable proof that tokens were destroyed
2. **Token Conversion**: Convert Promis to Gratis for different utility
3. **Governance**: Use burn tickets as voting power or participation proof
4. **Rewards**: Burn tokens to claim rewards or benefits
5. **Deflationary Mechanics**: Reduce token supply with permanent proof
6. **Multi-token Systems**: Bridge between different token utilities
7. **Achievement Systems**: Permanent proof of participation or milestones

## Conversion Economics

- **Rate**: 1 Promis = 1 Gratis (fixed)
- **Process**: Irreversible (cannot convert Gratis back to Promis)
- **Gas**: Single transaction handles both burn and mint
- **Atomicity**: Either both operations succeed or both fail

## Gas Optimization

- Efficient storage patterns using cw-storage-plus
- Minimal state updates during operations
- Optimized error handling
- Batch validation where possible
- Single cross-contract call for conversion

## Development

### Adding New Features
1. Update `msg.rs` with new message types
2. Implement handlers in `contract.rs`
3. Add corresponding tests
4. Update documentation

### Testing Guidelines
- Test all burn scenarios including edge cases
- Verify ticket generation and uniqueness
- Test conversion functionality thoroughly
- Validate security restrictions
- Test cross-contract communication
- Test error conditions

### Cross-contract Integration
When integrating with Gratis contract:
1. Ensure Gratis contract address is correct during instantiation
2. Verify Gratis contract has appropriate minter permissions
3. Test conversion in both success and failure scenarios

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
