use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Timestamp, Uint128};
use outbe_nft::state::NftInfo;
use outbe_nft::traits::{Cw721CollectionConfig, Cw721CustomMsg, Cw721State};
use outbe_utils::denom::Denom;

#[cw_serde]
pub struct Agent {
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
    pub avg_cu: u64,
    pub submitted_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cw_serde]
pub enum AgentType {
    NRA,
    CRA,
    RFA,
    IBA,
    CCA,
}

#[cw_serde]
pub enum AgentStatus {
    Pending,
    Approved,
    Rejected,
    Removed,
    OnHold,
    Blacklisted,
}
