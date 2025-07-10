# Token Minter Contract

A smart contract for minting Gratis and Promis tokens with access control list (ACL) functionality.

## Overview

This contract acts as a centralized minter for both Gratis and Promis tokens, implementing an access control list to manage who can mint which tokens. Only addresses explicitly added to the access list by the admin can mint tokens.

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

#### ExecuteMsg
- `Mint`: Mint tokens to a recipient (requires appropriate permissions)
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
let mint_msg = ExecuteMsg::Mint {
    recipient: "user_address".to_string(),
    amount: Uint128::from(1000u128),
    token_type: TokenType::Gratis,
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

## Security Features

- **Admin-only functions**: Critical operations restricted to contract admin
- **Granular permissions**: Separate permissions for each token type
- **Admin protection**: Admin cannot be removed from access list
- **Validation**: All addresses and amounts are validated before execution
- **Zero amount protection**: Prevents minting zero or negative amounts

## Deployment

1. Deploy Gratis and Promis token contracts first
2. Deploy this minter contract with their addresses
3. Set this contract as the minter for both token contracts
4. Add authorized addresses to the access list

## Testing

The contract includes comprehensive tests covering:
- Contract instantiation
- Token minting for both types
- Access control enforcement
- Admin functions
- Permission management
- Query functionality
- Error handling

Run tests with:
```bash
cargo test
```

## Schema Generation

Generate JSON schemas for integration:
```bash
cargo run --example schema
```