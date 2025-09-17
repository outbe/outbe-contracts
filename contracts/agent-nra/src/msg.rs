use crate::state::ThresholdConfig;
use crate::types::{Application, ApplicationInput, Vote};
use cosmwasm_schema::cw_serde;

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
pub enum ApplicationMsg {
    CreateApplication {
        application: Box<ApplicationInput>,
    },
    EditApplication {
        id: String,
        application:  Box<ApplicationInput>
    },
    VoteApplication {
        id: String,
        approve: bool,
        reason: Option<String>,
    },
    HoldApplication {
        id: String,
    },
    // BootstrapVote
    AddBootstrapVoter {
        address: String,
    },
    RemoveBootstrapVoter {
        address: String,
    },
}

pub type AgentMsg = agent_common::msg::ExecuteMsg;

#[cw_serde]
pub enum ExecuteMsg {
    Agent(AgentMsg),
    Application(ApplicationMsg),
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
