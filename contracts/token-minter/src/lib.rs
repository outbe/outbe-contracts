/// Smart contract for minting Gratis and Promis tokens with access control
/// 
/// This contract acts as a centralized minter for both Gratis and Promis tokens,
/// implementing an access control list (ACL) to manage who can mint which tokens.
/// Only addresses explicitly added to the access list by the admin can mint tokens.
/// 
/// Features:
/// - Mint Gratis and Promis tokens by calling their respective contracts
/// - Admin-managed access control list with granular permissions
/// - Query functions to check permissions and list authorized addresses
/// - Admin functions to manage the access list and transfer ownership

pub mod contract;
pub mod error;
pub mod msg;
pub mod query;
pub mod state;

#[cfg(test)]
mod tests;

pub use crate::error::ContractError;