use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128};
use strum_macros::Display;

#[cw_serde]
pub enum AgentStatus {
    Active,
    Blacklisted,
    InReview,
    OnHold,
    Resigned,
}

#[cw_serde]
pub enum AgentExt {
    Nra {},
    Cra {
        preferred_nra: Option<Vec<Addr>>,
        additional_wallets: Option<Vec<String>>,
    },
    Rfa {},
    Iba {
        preferred_nra: Option<Vec<Addr>>,
        additional_wallets: Option<Vec<ExternalWallet>>,
        license_number: Option<String>,
        license_uri: Option<String>,
    },
}

#[cw_serde]
#[derive(Display)]
pub enum AgentType {
    Nra,
    Cra,
    Rfa,
    Iba,
}
#[cw_serde]
pub struct Agent {
    pub wallet: Addr,
    pub agent_type: AgentType,
    pub name: String,
    pub email: Option<String>,
    pub jurisdictions: Vec<String>, // multi-select: ["eu","us",...]
    pub endpoint: Option<String>,
    pub metadata_json: Option<String>,
    pub docs_uri: Vec<String>,
    pub discord: Option<String>,
    pub status: AgentStatus,
    pub avg_cu: Option<Uint128>,
    pub submitted_at: Timestamp,
    pub updated_at: Timestamp,
    pub ext: AgentExt,
}

#[cw_serde]
pub struct AgentInput {
    pub name: String,
    pub email: Option<String>,
    pub jurisdictions: Vec<String>,
    pub endpoint: Option<String>,
    pub metadata_json: Option<String>,
    pub docs_uri: Vec<String>,
    pub discord: Option<String>,
    pub avg_cu: Option<Uint128>,
    pub ext: AgentExt,
}

#[cw_serde]
pub struct AgentDirectInput {
    pub name: String,
    pub email: Option<String>,
    pub jurisdictions: Vec<String>,
    pub endpoint: Option<String>,
    pub metadata_json: Option<String>,
    pub docs_uri: Vec<String>,
    pub discord: Option<String>,
    pub avg_cu: Option<Uint128>,
    pub ext: AgentExt,
    pub agent_type: AgentType,
}

#[cw_serde]
pub enum WalletType {
    Cosmos,
    Evm,
    Solana,
}

#[cw_serde]
pub struct ExternalWallet {
    pub wallet_type: WalletType,
    pub address: String,
}
