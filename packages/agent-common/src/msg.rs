use crate::types::Agent;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    pub application_registry_addr: Addr,
    pub paused: Option<bool>,
}
#[cw_serde]
pub struct AgentResponse {
    pub agent: Agent,
}
#[cw_serde]
pub struct ListAllAgentsResponse {
    pub agents: Vec<Agent>,
}
