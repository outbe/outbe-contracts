use crate::types::Deployment;
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Publish { deployment: Deployment },
    Ownable(cw_ownable::Action),
}
