use crate::types::{AgentInput, AgentStatus, ApplicationInput};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    pub threshold: Option<u8>,
    pub paused: Option<bool>,
}

#[cw_serde]
pub enum MigrateMsg {
    Migrate {},
}

#[cw_serde]
pub enum ExecuteMsg {
    // Application
    CreateApplication {
        application: ApplicationInput,
    },
    EditApplication {
        id: String,
        application: ApplicationInput,
    },
    VoteApplication {
        id: String,
        approve: bool,
        reason: Option<String>,
    },
    HoldApplication {
        id: String,
    },

    RejectApplication {
        id: String,
    },


    // Agent
    SubmitAgent {
    },

    EditAgent {
    },

    HoldAgent {
        agent_id: String,
    },
    BanAgent {
        agent_id: String,
    },
    ActivateAgent {
        agent_id: String,
    },

    ResignAgent {
    },

}
