use agent_common::types::AgentInput;
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum MigrateMsg {
    Migrate {},
}

pub type ExecuteMsg = agent_common::msg::ExecuteMsg;
