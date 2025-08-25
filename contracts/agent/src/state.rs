use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use crate::types::{Agent, Config, Vote};

pub const AGENTS: Map<String,Agent> = Map::new("agents");
pub const CONFIG: Item<Config> = Item::new("config");

pub const AGENT_VOTES: Map<(&str, &Addr), Vote> = Map::new("agent_votes");
