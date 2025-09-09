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

    #[error("only the wallet owner can create agent-nra")]
    OwnerError {},

    #[error("agent-nra not found")]
    ApplicationNotFound {},

    #[error("only the application owner can submit agent")]
    ApplicationOwnerError {},

    #[error("application not approved")]
    ApplicationNotApproved {},

    #[error("invalid application type")]
    ApplicationInvalidType {},
    #[error("Invalid application status for this operation")]
    InvalidApplicationStatus {},

    // Vote
    #[error("only active NRA can vote")]
    OnlyActiveNra {},

    #[error("applicant cannot vote on own record")]
    SelfVote {},

    #[error("already voted")]
    AlreadyVoted {},

    #[error("already finalized")]
    AlreadyFinalized {},
    // Agent
    #[error("Agent not found")]
    AgentNotFound {},
    #[error("Invalid agent status for this operation")]
    InvalidAgentStatus {},

    //Botstrap voterrs
    #[error("Invalid action")]
    InvalidBootstrapAction {},
}
