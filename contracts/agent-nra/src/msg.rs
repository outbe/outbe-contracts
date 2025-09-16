use crate::types::{Application, ApplicationInput, ThresholdConfig, Vote};
use cosmwasm_schema::cw_serde;
use agent_common::types::AgentInput;

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
        agent: AgentInput,
    },

    HoldAgent {
        address: String,
    },
    BanAgent {
        address: String,
    },
    ActivateAgent {
        address: String,
    },

    ResignAgent {},

    // BootstrapVote
    AddBootstrapVoter {
        address: String,
    },
    RemoveBootstrapVoter {
        address: String,
    },
}

#[cw_serde]
pub struct ListAllApplicationResponse {
    pub applications: Vec<Application>,
}

#[cw_serde]
pub struct ApplicationResponse {
    pub application: Option<Application>,
}

#[cw_serde]
pub struct ApplicationVotesResponse {
    pub votes: Vec<Vote>,
}


#[cw_serde]
pub struct NraAccessResponse {
    pub allowed: bool,
}