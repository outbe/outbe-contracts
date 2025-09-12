use crate::types::Config;
use agent_nra::types::Agent;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const CONFIG: Item<Config> = Item::new("config");

pub const AGENTS: Map<Addr, Agent> = Map::new("agents");
