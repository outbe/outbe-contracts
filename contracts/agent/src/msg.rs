use crate::types::AgentInput;
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
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
