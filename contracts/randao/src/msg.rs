use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    pub creator: Option<String>,
    pub seed: u64,
}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
pub enum MigrateMsg {
    Migrate {},
}
