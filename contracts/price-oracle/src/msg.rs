use crate::types::TokenPairPrice;
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    pub creator: Option<String>,
    pub initial_price: TokenPairPrice,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdatePrice { token_pair_price: TokenPairPrice },
}

#[cw_serde]
pub enum MigrateMsg {
    Migrate {},
}
