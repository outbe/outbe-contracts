use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

use crate::state::{AccessPermissions, Config, TokenType};

/// Message to instantiate the contract
#[cw_serde]
pub struct InstantiateMsg {
    /// Address of the Gratis token contract
    pub gratis_contract: String,
    /// Address of the Promis token contract
    pub promis_contract: String,
    /// Address of the Price Oracle contract
    pub price_oracle_contract: String,
    /// Address of the Nod NFT contract
    pub nod_contract: String,
    /// Initial access
    pub access_list: Vec<AccessMsg>,
}

#[cw_serde]
pub struct AccessMsg {
    /// Address to add to the access list
    pub address: String,
    /// Permissions for this address
    pub permissions: AccessPermissions,
}

/// Execute messages for the contract
#[cw_serde]
pub enum ExecuteMsg {
    /// Mint tokens to a recipient address
    /// Only addresses in the access list with appropriate permissions can call this
    Mine {
        /// Address to receive the minted tokens
        recipient: String,
        /// Amount of tokens to mint
        amount: Uint128,
        /// Type of token to mint (Gratis or Promis)
        token_type: TokenType,
    },
    /// Mine Gratis tokens using a qualified Nod NFT
    /// This will check if the current price from Price Oracle is >= floor_price_minor
    /// If qualified, it will mint Gratis tokens based on gratis_load_minor and burn the Nod NFT
    MineGratisWithNod {
        /// Token ID of the Nod NFT to use for mining
        nod_token_id: String,
    },
    /// Add an address to the access list (admin only)
    AddToAccessList {
        /// Address to add to the access list
        address: String,
        /// Permissions for this address
        permissions: AccessPermissions,
    },
    /// Remove an address from the access list (admin only)
    RemoveFromAccessList {
        /// Address to remove from the access list
        address: String,
    },
    /// Update permissions for an existing address in the access list (admin only)
    UpdatePermissions {
        /// Address to update permissions for
        address: String,
        /// New permissions for this address
        permissions: AccessPermissions,
    },
    /// Transfer admin rights to a new address (admin only)
    TransferAdmin {
        /// New admin address
        new_admin: String,
    },
    /// Update contract addresses (admin only)
    UpdateContracts {
        /// New Gratis contract address (optional)
        gratis_contract: Option<String>,
        /// New Promis contract address (optional)
        promis_contract: Option<String>,
        /// New Price Oracle contract address (optional)
        price_oracle_contract: Option<String>,
        /// New Nod NFT contract address (optional)
        nod_contract: Option<String>,
    },
}

/// Query messages for the contract
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get contract configuration
    #[returns(ConfigResponse)]
    Config {},
    /// Get access permissions for a specific address
    #[returns(AccessPermissionsResponse)]
    AccessPermissions { address: String },
    /// List all addresses in the access list with optional pagination
    #[returns(AccessListResponse)]
    AccessList {
        /// Address to start listing from (optional, for pagination)
        start_after: Option<String>,
        /// Maximum number of addresses to return
        limit: Option<u32>,
    },
    /// Check if an address can mint a specific token type
    #[returns(CanMintResponse)]
    CanMint {
        /// Address to check
        address: String,
        /// Token type to check
        token_type: TokenType,
    },
}

/// Response for Config query
#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}

/// Response for AccessPermissions query
#[cw_serde]
pub struct AccessPermissionsResponse {
    /// The address being queried
    pub address: Addr,
    /// Permissions for this address (None if not in access list)
    pub permissions: Option<AccessPermissions>,
}

/// Response for AccessList query
#[cw_serde]
pub struct AccessListResponse {
    /// List of addresses and their permissions
    pub addresses: Vec<(Addr, AccessPermissions)>,
}

/// Response for CanMint query
#[cw_serde]
pub struct CanMintResponse {
    /// Whether the address can mint the specified token type
    pub can_mint: bool,
    /// Human-readable reason if cannot mint
    pub reason: Option<String>,
}
