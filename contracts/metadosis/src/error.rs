use cosmwasm_std::StdError;
use outbe_utils::date::DateError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),
    #[error("Not initialized")]
    NotInitialized {},
    #[error("Bad Run Configuration")]
    BadRunConfiguration {},
    #[error(transparent)]
    DateError(#[from] DateError),
}
