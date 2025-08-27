use cosmwasm_std::{StdError, VerificationError};
use cw_ownable::OwnershipError;
use outbe_utils::amount_utils::AmountError;
use outbe_utils::date::DateError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),
    #[error(transparent)]
    VerificationError(#[from] VerificationError),
    #[error(transparent)]
    Ownership(#[from] OwnershipError),
    #[error(transparent)]
    AmountError(#[from] AmountError),
    #[error("Not initialized")]
    NotInitialized {},
    #[error("ID already exists")]
    IdAlreadyExists {},
    #[error("Consumption Unit already exists")]
    CUAlreadyExists {},
    #[error("Consumption Units are empty")]
    CUEmpty {},
    #[error(transparent)]
    DateError(#[from] DateError),
    #[error("Provided tribute_draft_id does not match expected value")]
    InvalidDraftId {},
    #[error("Invalid encryption key")]
    InvalidKey {},
    #[error("Invalid nonce")]
    InvalidNonce {},
    #[error("Decryption failed")]
    DecryptionFailed {},
    #[error("Invalid payload format")]
    InvalidPayload {},
    #[error("Invalid TEE configuration")]
    InvalidTeeConfig {},
    #[error("Public key does not match private key")]
    InvalidKeyPair {},
    #[error("Invalid salt length")]
    InvalidSalt {},
}
