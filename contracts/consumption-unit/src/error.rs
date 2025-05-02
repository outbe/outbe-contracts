use cosmwasm_std::StdError;
use q_nft::error::Cw721ContractError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("{0}")]
    Cw721ContractError(#[from] Cw721ContractError),
    #[error("WrongInput")]
    WrongInput {},
    #[error("WrongDigest")]
    WrongDigest {},
    #[error("WrongTier")]
    WrongTier {},
    #[error("HashAlreadyExists")]
    HashAlreadyExists {},
}
