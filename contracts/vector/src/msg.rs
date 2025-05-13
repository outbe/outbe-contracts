use crate::types::Vector;
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    pub vectors: Option<Vec<Vector>>,
    pub creator: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {}
