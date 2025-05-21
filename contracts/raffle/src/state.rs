use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_ownable::{OwnershipStore, OWNERSHIP_KEY};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub vector: Option<Addr>,
    pub tribute: Option<Addr>,
    pub nod: Option<Addr>,
    pub token_allocator: Option<Addr>,
    pub price_oracle: Option<Addr>,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const CREATOR: OwnershipStore = OwnershipStore::new(OWNERSHIP_KEY);

#[cw_serde]
pub struct RaffleRunData {
    pub raffle_date: Timestamp,
    pub raffle_date_time: Timestamp,
    pub pool_id: u16,
    pub tributes_in_pool: usize,
    pub total_allocation: Uint128,
    pub allocation_per_pool: Uint128,
    pub pool_capacity: Uint128,
    pub allocated_in_pool: Uint128,
}

#[cw_serde]
pub struct RaffleHistory {
    pub data: Vec<RaffleRunData>,
}

// todo demo only
pub const HISTORY: Item<RaffleHistory> = Item::new("history");

pub const DAILY_RAFFLE: Map<u64, u16> = Map::new("daily_raffle");
pub const TRIBUTES_DISTRIBUTION: Map<&str, String> = Map::new("tributes_distribution");
