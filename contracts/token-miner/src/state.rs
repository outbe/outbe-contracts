use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Configuration for the token minter contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// Address of the admin who can manage the access list
    pub admin: Addr,
    /// Address of the Gratis token contract
    pub gratis_contract: Addr,
    /// Address of the Promis token contract
    pub promis_contract: Addr,
}

/// Token types that can be minted
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum TokenType {
    Gratis,
    Promis,
}

/// Access permissions for an address
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AccessPermissions {
    /// Whether this address can mint Gratis tokens
    pub can_mint_gratis: bool,
    /// Whether this address can mint Promis tokens
    pub can_mint_promis: bool,
    /// Optional note about this address (for admin reference)
    pub note: Option<String>,
}

/// Contract configuration storage
pub const CONFIG: Item<Config> = Item::new("config");

/// Access control list mapping addresses to their permissions
/// Key: Addr (the address), Value: AccessPermissions
pub const ACCESS_LIST: Map<&Addr, AccessPermissions> = Map::new("access_list");
