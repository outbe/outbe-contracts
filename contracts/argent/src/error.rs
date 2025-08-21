use cosmwasm_std::StdError;
use outbe_nft::error::Cw721ContractError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
}
