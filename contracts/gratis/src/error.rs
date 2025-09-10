use cosmwasm_std::StdError;
use cw20_base::ContractError as Cw20ContractError;
use serde_json_wasm::ser::Error as SerdeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),
    #[error(transparent)]
    Cw20Error(#[from] Cw20ContractError),
    #[error(transparent)]
    SerdeError(#[from] SerdeError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("User already burned tokens in this block")]
    AlreadyBurnedInBlock {},

    #[error("Contract has insufficient native funds")]
    InsufficientContractFunds {},
}
