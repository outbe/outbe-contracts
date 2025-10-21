use cosmwasm_std::{StdError};
use thiserror::Error;

/// Custom error types for the token minter contract
#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    /// Unauthorized access - sender is not the admin
    #[error("Unauthorized: only admin can perform this action")]
    Unauthorized {},

    /// Address not found in access list
    #[error("Address not found in access list")]
    AddressNotInAccessList {},

    /// Address already exists in access list
    #[error("Address already exists in access list")]
    AddressAlreadyInAccessList {},

    /// No permission to mint the specified token type
    #[error("No permission to mint {token_type:?} tokens")]
    NoMintPermission { token_type: String },

    /// Invalid token amount (zero or negative)
    #[error("Invalid token amount: must be greater than zero")]
    InvalidAmount {},

    /// Invalid address format
    #[error("Invalid address format")]
    InvalidAddress {},

    /// Contract address cannot be changed to the same address
    #[error("New contract address is the same as current address")]
    SameContractAddress {},

    /// Cannot remove admin from access list
    #[error("Cannot remove admin from access list")]
    CannotRemoveAdmin {},

    /// Cannot transfer admin to the same address
    #[error("Cannot transfer admin to the same address")]
    SameAdminAddress {},

    /// Sender is not the owner of the Nod NFT
    #[error("Not authorized: sender is not the owner of the Nod NFT")]
    NotNodOwner {},

    /// Invalid proof-of-work
    #[error("Invalid proof-of-work")]
    InvalidProofOfWork {},
    #[error("Invalid hash")]
    InvalidHash {},

    /// Nod NFT is not qualified for mining (current price < floor price)
    #[error("Nod NFT is not qualified for mining")]
    NodNotQualified {},
}
