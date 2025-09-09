use agent_nra::types::AgentInput;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    pub agent_registry: Addr,
    pub paused: Option<bool>,
}

#[cw_serde]
pub enum MigrateMsg {
    Migrate {},
}

#[cw_serde]
pub enum ExecuteMsg {
    // Agent
    SubmitAgent { id: String },

    EditAgent { agent: AgentInput },

    HoldAgent { address: String },
    BanAgent { address: String },
    ActivateAgent { address: String },

    ResignAgent {},
}
