use crate::types::{AgentInput, AgentStatus, AgentType, Vote};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Addr,
    pub threshold: Option<u8>,
    pub paused: Option<bool>,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateAgent {
        agent: AgentInput,
    },
    UpdateAgent {
        id: String,
        agent: AgentInput,
    },
    VoteAgent {
        id: String,
        approve: bool,
        reason: Option<String>,
    },
}
