use cosmwasm_std::StdError;
use cw_utils::ParseReplyError;
use outbe_utils::date::{DateError, WorldwideDay};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),
    #[error("Not initialized")]
    NotInitialized {},
    #[error("Not Prepared for run")]
    NotPrepared {},
    #[error("Bad Run Configuration")]
    BadRunConfiguration {},
    #[error("Data already prepared {day}")]
    AlreadyPrepared { day: WorldwideDay },
    #[error(transparent)]
    DateError(#[from] DateError),
    #[error("Bad Reply ID {id}")]
    UnrecognizedReplyId { id: u64 },
    #[error("NoDataInReply")]
    NoDataInReply {},
    #[error(transparent)]
    ParseReplyError(#[from] ParseReplyError),
}
