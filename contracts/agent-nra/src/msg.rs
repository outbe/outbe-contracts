use crate::types::{AgentInput, ApplicationInput, ThresholdConfig};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    pub thresholds: Option<ThresholdConfig>,
    pub paused: Option<bool>,
    pub bootstrap_voters: Option<Vec<String>>,

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



    // Agent
    SubmitAgent {
        id: String,
    },

    EditAgent {
        agent:AgentInput,
    },

    HoldAgent {
        address: String
    },
    BanAgent {
        address: String
    },
    ActivateAgent {
        address: String
    },

    ResignAgent {
    },

}
