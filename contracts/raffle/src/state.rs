use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
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

pub const DAILY_RAFFLE: Map<u64, u16> = Map::new("daily_raffle");
pub const TRIBUTES_DISTRIBUTION: Map<&str, String> = Map::new("tributes_distribution");
