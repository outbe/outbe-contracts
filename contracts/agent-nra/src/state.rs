use crate::types::{Application, Vote};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub thresholds: ThresholdConfig,
    pub paused: bool,
    pub last_application_id: u32,
    pub bootstrap_voters: Vec<Addr>,
}

#[cw_serde]
pub struct ThresholdConfig {
    pub nra: u8,
    pub cra: u8,
    pub rfa: u8,
    pub iba: u8,
    pub cca: u8,
}
pub const APPLICATIONS: Map<String, Application> = Map::new("applications");
pub const CONFIG: Item<Config> = Item::new("config");

pub const APPLICATION_VOTES: Map<(&str, &Addr), Vote> = Map::new("application_votes");
