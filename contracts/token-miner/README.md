# Token Miner Contract

A smart contract for minting Gratis and Promis tokens with access control list (ACL) functionality, and Nod-based Gratis mining capabilities.

## Overview

This contract acts as a centralized miner for both Gratis and Promis tokens, implementing:
1. **Access Control List (ACL)** - Managing who can mint which tokens
2. **Nod-based Gratis Mining** - Mining Gratis tokens using qualified Nod NFTs with price oracle validation
3. **Admin Management** - Comprehensive administrative controls

The contract integrates with Price Oracle and Nod NFT contracts to enable price-qualified Gratis mining.

## Features

- **Token Minting**: Mint Gratis and Promis tokens by calling their respective contracts
- **Access Control**: Admin-managed access control list with granular permissions per token type
- **Permission Management**: Grant specific permissions (Gratis only, Promis only, or both)
- **Admin Functions**: Manage the access list, update contract addresses, and transfer ownership
- **Query Interface**: Check permissions, list authorized addresses, and verify minting capabilities

## Contract Structure

### State

- **Config**: Stores admin address and token contract addresses
- **Access List**: Maps addresses to their specific permissions

### Messages

#### InstantiateMsg
- `gratis_contract`: Address of the Gratis token contract
- `promis_contract`: Address of the Promis token contract
- `price_oracle_contract`: Address of the Price Oracle contract
- `nod_contract`: Address of the Nod NFT contract

#### ExecuteMsg
- `Mine`: Mint tokens to a recipient (requires appropriate permissions)
- `MineGratisWithNod`: Mine Gratis tokens using a qualified Nod NFT (owner only)
- `AddToAccessList`: Add an address to the access list (admin only)
- `RemoveFromAccessList`: Remove an address from access list (admin only)
- `UpdatePermissions`: Update permissions for an existing address (admin only)
- `TransferAdmin`: Transfer admin rights to a new address (admin only)
- `UpdateContracts`: Update token contract addresses (admin only)

#### QueryMsg
- `Config`: Get contract configuration
- `AccessPermissions`: Get permissions for a specific address
- `AccessList`: List all addresses in access list with pagination
- `CanMint`: Check if an address can mint a specific token type

## Access Control

Each address in the access list has granular permissions:

```rust
pub struct AccessPermissions {
    pub can_mint_gratis: bool,    // Can mint Gratis tokens
    pub can_mint_promis: bool,    // Can mint Promis tokens
    pub note: Option<String>,     // Optional note for admin reference
}
```

## Usage Examples

### Minting Tokens

```rust
// Mint 1000 Gratis tokens to user
let mint_msg = ExecuteMsg::Mine {
    recipient: "user_address".to_string(),
    amount: Uint128::from(1000u128),
    token_type: TokenType::Gratis,
};
```

### Nod-based Gratis Mining

```rust
// Mine Gratis tokens using a qualified Nod NFT
let mine_msg = ExecuteMsg::MineGratisWithNod {
    nod_token_id: "nod_123".to_string(),
};
```

### Managing Access List

```rust
// Add user with Gratis-only permissions
let permissions = AccessPermissions {
    can_mint_gratis: true,
    can_mint_promis: false,
    note: Some("Service account".to_string()),
};
let add_msg = ExecuteMsg::AddToAccessList {
    address: "service_address".to_string(),
    permissions,
};
```

### Querying Permissions

```rust
// Check if address can mint Promis tokens
let query_msg = QueryMsg::CanMint {
    address: "user_address".to_string(),
    token_type: TokenType::Promis,
};
```

## Nod-based Gratis Mining Process

1. **Price Qualification**: Current price from Price Oracle must be >= Nod's floor_price_minor
2. **State Verification**: Nod NFT must be in "Issued" state
3. **Ownership Check**: Only the Nod owner can initiate mining
4. **Gratis Minting**: Amount minted equals Nod's gratis_load_minor value
5. **Nod Burning**: Nod NFT is automatically burned after successful mining

## Security Features

- **Admin-only functions**: Critical operations restricted to contract admin
- **Granular permissions**: Separate permissions for each token type
- **Admin protection**: Admin cannot be removed from access list
- **Validation**: All addresses and amounts are validated before execution
- **Zero amount protection**: Prevents minting zero or negative amounts
- **Ownership verification**: Nod-based mining requires NFT ownership
- **Price validation**: Oracle price must meet qualification threshold

## Deployment

1. Deploy Gratis and Promis token contracts first
2. Deploy Price Oracle and Nod NFT contracts
3. Deploy this miner contract with all four contract addresses
4. Set this contract as the miner for both token contracts
5. Add authorized addresses to the access list

## Testing

The contract includes 18 comprehensive tests covering:
- Contract instantiation
- Token minting for both types
- Access control enforcement
- Admin functions
- Permission management
- Query functionality
- Error handling
- Nod-based Gratis mining success scenarios
- Nod-based mining error cases (ownership, state, price qualification)

Run tests with:
```bash
cargo test
```

## Schema Generation

Generate JSON schemas for integration:
```bash
cargo run --example schema
```