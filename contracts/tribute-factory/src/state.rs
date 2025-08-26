use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Empty};
use cw_ownable::{OwnershipStore, OWNERSHIP_KEY};
use cw_storage_plus::{Item, Map};
use outbe_utils::Base58Binary;

pub const OWNER: OwnershipStore = OwnershipStore::new(OWNERSHIP_KEY);

#[cw_serde]
pub struct Config {
    pub tribute_address: Option<Addr>,
    pub tee_config: Option<TeeConfig>,
}

#[cw_serde]
pub struct TeeConfig {
    pub private_key: Base58Binary,
    pub public_key: Base58Binary,
    pub salt: Base58Binary,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const USED_TRIBUTE_IDS: Map<String, Empty> = Map::new("used_ids");

pub const USED_CU_HASHES: Map<String, Empty> = Map::new("used_cu_hashes");
