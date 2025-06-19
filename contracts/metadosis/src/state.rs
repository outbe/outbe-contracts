use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_ownable::{OwnershipStore, OWNERSHIP_KEY};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub vector: Option<Addr>,
    pub tribute: Option<Addr>,
    pub nod: Option<Addr>,
    pub token_allocator: Option<Addr>,
    pub price_oracle: Option<Addr>,
    pub deficit: Decimal,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const CREATOR: OwnershipStore = OwnershipStore::new(OWNERSHIP_KEY);

#[cw_serde]
pub enum RunType {
    Lysis,
    Touch,
}

#[cw_serde]
pub struct RaffleRunInfo {
    pub vector_index: u16,
    pub run_type: RunType,

    pub total_allocation: Uint128,
    pub pool_allocation: Uint128,
    pub total_deficit: Uint128,
    pub pool_deficit: Uint128,
    pub pool_capacity: Uint128,
    pub assigned_tributes: usize,
    pub assigned_tributes_sum: Uint128,
}

#[cw_serde]
pub struct DailyRaffleRunInfo {
    pub data: Vec<RaffleRunInfo>,
    pub number_of_runs: usize,
}

pub const DAILY_RAFFLE_RUN: Map<u64, usize> = Map::new("daily_raffle_run");
pub const DAILY_RAFFLE_RUN_INFO: Map<u64, DailyRaffleRunInfo> = Map::new("daily_raffle_run_info");

pub const TRIBUTES_DISTRIBUTION: Map<&str, String> = Map::new("tributes_distribution");
