use cosmwasm_schema::cw_serde;
use outbe_utils::date::WorldwideDay;

#[cw_serde]
pub struct InstantiateMsg {
    pub creator: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    AllocateTokens { date: WorldwideDay },
}

#[cw_serde]
pub enum MigrateMsg {
    Migrate {},
}
