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
}
