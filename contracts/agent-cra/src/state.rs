use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use agent_nra::types::{Agent};
use crate::types::Config;

pub const CONFIG: Item<Config> = Item::new("config");


pub const AGENTS: Map<Addr, Agent> = Map::new("agents");
