use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub thresholds: ThresholdConfig,
    pub paused: bool,
    pub last_token_id: u32,
    pub bootstrap_voters: Vec<Addr>,
}

#[cw_serde]
pub struct ThresholdConfig {
    pub nra: u8,
    pub cra: u8,
    pub rfa: u8,
    pub iba: u8,
    pub cca: u8,
}

#[cw_serde]
pub struct Application {
    pub id: u32,
    pub application_type: ApplicationType,
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
    pub ext: Option<ApplicationExt>,
}

#[cw_serde]
pub struct ApplicationInput {
    pub application_type: ApplicationType,
    pub name: String,
    pub email: String,
    pub jurisdictions: Vec<String>,
    pub endpoint: Option<String>,
    pub metadata_json: Option<String>,
    pub docs_uri: Vec<String>,
    pub discord: Option<String>,
    pub avg_cu: Option<Uint128>,
    pub ext: Option<ApplicationExt>,
}

#[cw_serde]

pub enum ApplicationExt {
    Nra {},
    Cra { preferred_nra: Option<Vec<String>> },
    Rfa {},
    Iba {},
    Cca {},
}

#[cw_serde]
pub enum ApplicationType {
    Nra,
    Cra,
    Rfa,
    Iba,
    Cca,
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
pub enum AgentStatus {
    Active,
    Blacklisted,
    InReview,
    OnHold,
    Resigned,
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
pub struct Vote {
    pub address: String,
    pub application_id: String,
    pub approve: bool,
    pub reason: Option<String>,
    pub at: Timestamp,
}

#[cw_serde]
pub struct ApplicationVotesResponse {
    pub votes: Vec<Vote>,
}

#[cw_serde]
pub struct Agent {
    pub wallet: Addr,
    pub agent_type: ApplicationType,
    pub name: String,
    pub email: String,
    pub jurisdictions: Vec<String>, // multi-select: ["eu","us",...]
    pub endpoint: Option<String>,
    pub metadata_json: Option<String>,
    pub docs_uri: Vec<String>,
    pub discord: Option<String>,
    pub status: AgentStatus,
    pub avg_cu: Option<Uint128>,
    pub submitted_at: Timestamp,
    pub updated_at: Timestamp,
    pub ext: ApplicationExt,
}

#[cw_serde]
pub struct AgentInput {
    pub name: String,
    pub email: String,
    pub jurisdictions: Vec<String>,
    pub endpoint: Option<String>,
    pub metadata_json: Option<String>,
    pub docs_uri: Vec<String>,
    pub discord: Option<String>,
    pub avg_cu: Option<Uint128>,
    pub ext: ApplicationExt,
}

#[cw_serde]
pub struct AgentResponse {
    pub agent: Agent,
}

#[cw_serde]
pub struct ListAllAgentsResponse {
    pub agents: Vec<Agent>,
}

#[cw_serde]
pub struct NraAccessResponse {
    pub allowed: bool,
}
