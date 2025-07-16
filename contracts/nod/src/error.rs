use cosmwasm_std::StdError;
use outbe_nft::error::Cw721ContractError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("{0}")]
    Cw721ContractError(#[from] Cw721ContractError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Claim condition not met: floor price must be lower than oracle price")]
    ClaimConditionNotMet {},
}
