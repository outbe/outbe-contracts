use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub paused: bool,
    pub last_token_id: u32,
    pub agent_registry: Addr,

}

