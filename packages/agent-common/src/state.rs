use crate::types::Agent;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub paused: bool,
    pub last_token_id: u32,
    pub agent_registry: Addr,
}
pub const CONFIG: Item<Config> = Item::new("config");

pub const AGENTS: Map<Addr, Agent> = Map::new("agents");
