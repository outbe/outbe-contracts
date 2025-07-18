use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

/// Configuration for the token minter contract
#[cw_serde]
pub struct Config {
    /// Address of the admin who can manage the access list
    pub admin: Addr,
    /// Address of the Gratis token contract
    pub gratis_contract: Addr,
    /// Address of the Promis token contract
    pub promis_contract: Addr,
    /// Address of the Price Oracle contract
    pub price_oracle_contract: Addr,
    /// Address of the Nod NFT contract
    pub nod_contract: Addr,
}

/// Token types that can be minted
#[cw_serde]
pub enum TokenType {
    Gratis,
    Promis,
}

/// Access permissions for an address
#[cw_serde]
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
