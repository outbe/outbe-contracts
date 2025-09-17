use agent_common::types::AgentInput;
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum MigrateMsg {
    Migrate {},
}

#[cw_serde]
pub enum ExecuteMsg {
    // Agent
    SubmitAgent { id: String },

    EditAgent { agent: Box<AgentInput> },

    HoldAgent { address: String },
    BanAgent { address: String },
    ActivateAgent { address: String },

    ResignAgent {},
}
