use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub threshold: u8,
    pub paused: bool,
    pub last_token_id: u32,
}

#[cw_serde]
pub struct Agent {
    pub id: u32,
    pub agent_type: AgentType,
    pub wallet: Addr,
    pub name: String,
    pub email: String,
    pub jurisdictions: Vec<String>, // multi-select: ["eu","us",...]
    pub endpoint: Option<String>,
    pub metadata_json: Option<String>,
    pub docs_uri: Vec<String>,
    pub discord: Option<String>,
    pub status: AgentStatus,
    pub avg_cu: Uint128,
    pub submitted_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cw_serde]
pub struct AgentInput {
    pub agent_type: AgentType,
    pub name: String,
    pub email: String,
    pub jurisdictions: Vec<String>,
    pub endpoint: Option<String>,
    pub metadata_json: Option<String>,
    pub docs_uri: Vec<String>,
    pub discord: Option<String>,
    pub status: AgentStatus,
    pub avg_cu: Uint128,
}

#[cw_serde]
pub enum AgentType {
    Nra,
    Cra,
    Rfa,
    Iba,
    Cca,
}

#[cw_serde]
pub enum AgentStatus {
    Pending,
    Approved,
    Rejected,
    Recalled,
}

#[cw_serde]
pub enum AccountStatus {
    Approved,
    Blacklisted,
    OnHold,
}

#[cw_serde]
pub struct ListAllResponse {
    pub agents: Vec<Agent>,
}

#[cw_serde]
pub struct AgentResponse {
    pub agent: Agent,
}

#[cw_serde]
pub struct Vote {
    pub address: String,
    pub approve: bool,
    pub reason: Option<String>,
    pub at: Timestamp,
}

#[cw_serde]
pub struct AgentVotesResponse {
    pub votes: Vec<Vote>,
}

#[cw_serde]
pub struct Account {
    pub agent_type: AgentType,
    pub name: String,
    pub email: String,
    pub jurisdictions: Vec<String>, // multi-select: ["eu","us",...]
    pub endpoint: Option<String>,
    pub metadata_json: Option<String>,
    pub docs_uri: Vec<String>,
    pub discord: Option<String>,
    pub status: AgentStatus,
    pub avg_cu: Uint128,
    pub submitted_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cw_serde]
pub struct AccountInput {
    pub name: String,
    pub email: String,
    pub jurisdictions: Vec<String>,
    pub endpoint: Option<String>,
    pub metadata_json: Option<String>,
    pub docs_uri: Vec<String>,
    pub discord: Option<String>,
    pub avg_cu: Uint128,
}

#[cw_serde]
pub struct AccountResponse {
    pub account: Account,
}

#[cw_serde]
pub struct ListAllAccountsResponse {
    pub accounts: Vec<Account>,
}
