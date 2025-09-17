use agent_common::types::{AgentExt, AgentType};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128};
#[cw_serde]
pub struct Application {
    pub id: u32,
    pub application_type: AgentType,
    pub wallet: Addr,
    pub name: String,
    pub email: String,
    pub endpoint: Option<String>,
    pub discord: Option<String>,
    pub jurisdictions: Vec<String>, // multi-select: ["eu","us",...]
    pub docs_uri: Vec<String>,
    pub metadata_json: Option<String>,
    pub status: ApplicationStatus,
    pub avg_cu: Option<Uint128>,
    pub submitted_at: Timestamp,
    pub updated_at: Timestamp,
    pub ext: Option<AgentExt>,
}

#[cw_serde]
pub struct ApplicationInput {
    pub application_type: AgentType,
    pub name: String,
    pub email: String,
    pub jurisdictions: Vec<String>,
    pub endpoint: Option<String>,
    pub metadata_json: Option<String>,
    pub docs_uri: Vec<String>,
    pub discord: Option<String>,
    pub avg_cu: Option<Uint128>,
    pub ext: Option<AgentExt>,
}

#[cw_serde]
pub enum ApplicationStatus {
    InReview,
    Approved,
    OnHold,
    Rejected,
    Recalled,
}

#[cw_serde]
pub struct Vote {
    pub address: String,
    pub application_id: String,
    pub approve: bool,
    pub reason: Option<String>,
    pub at: Timestamp,
}
