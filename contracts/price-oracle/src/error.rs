use cosmwasm_std::StdError;
use cw_ownable::OwnershipError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Ownership(#[from] OwnershipError),

    #[error("Token pair already exists: {pair_id}")]
    PairAlreadyExists { pair_id: String },

    #[error("Token pair not found: {pair_id}")]
    PairNotFound { pair_id: String },

    #[error("TLast price for {pair_id}, not found ")]
    LatestPriceNotFound { pair_id: String },

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid token pair: tokens must be different")]
    InvalidTokenPair {},

    #[error("Price history is empty for pair: {pair_id}")]
    EmptyPriceHistory { pair_id: String },

    #[error("Invalid time range: start_time must be before end_time")]
    InvalidTimeRange {},

    #[error("Day type not found for pair: {pair_id}")]
    DayTypeNotFound { pair_id: String },

    #[error("VWAP not available for pair: {pair_id}")]
    VwapNotAvailable { pair_id: String },
}
