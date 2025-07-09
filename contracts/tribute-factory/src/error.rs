use cosmwasm_std::{StdError, VerificationError};
use cw_ownable::OwnershipError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),
    #[error(transparent)]
    VerificationError(#[from] VerificationError),
    #[error(transparent)]
    Ownership(#[from] OwnershipError),
    #[error("Not initialized")]
    NotInitialized {},
    #[error("ID already exists")]
    IdAlreadyExists {},
    #[error("Consumption Unit already exists")]
    CUAlreadyExists {},
}
