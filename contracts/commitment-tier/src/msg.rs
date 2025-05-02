use crate::types::CommitmentTier;
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    pub tiers: Option<Vec<CommitmentTier>>,
    pub creator: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {}
