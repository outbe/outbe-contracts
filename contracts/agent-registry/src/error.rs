use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error("contract is paused")]
    Paused,

    #[error("unauthorized")]
    Unauthorized,

    #[error("only the wallet owner can create agent-registry")]
    OwnerError {},

    #[error("agent-registry not found")]
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
