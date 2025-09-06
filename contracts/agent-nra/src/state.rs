use crate::types::{Agent, Application, Config, Vote};
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const APPLICATIONS: Map<String, Application> = Map::new("applications");
pub const CONFIG: Item<Config> = Item::new("config");

pub const APPLICATION_VOTES: Map<(&str, &Addr), Vote> = Map::new("application_votes");

pub const AGENTS: Map<Addr, Agent> = Map::new("agents");
