use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("contract is paused")]
    Paused,

    #[error("unauthorized")]
    Unauthorized,

    #[error("only the wallet owner can create agent")]
    OwnerError {},

    #[error("agent not found")]
    AgentNotFound {},

    #[error("only active NRA can vote")]
    OnlyActiveNra {},
    
    #[error("applicant cannot vote on own record")]
    SelfVote {},
    
    #[error("already voted")]
    AlreadyVoted {},

    #[error("already finalized")]
    AlreadyFinalized {},
}
