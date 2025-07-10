use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Empty, HexBinary};
use cw_ownable::{OwnershipStore, OWNERSHIP_KEY};
use cw_storage_plus::{Item, Map};

pub const OWNER: OwnershipStore = OwnershipStore::new(OWNERSHIP_KEY);

#[cw_serde]
pub struct Config {
    pub tribute_address: Option<Addr>,
    pub tee_config: Option<TeeConfig>,
}

#[cw_serde]
pub struct TeeConfig {
    pub private_key: HexBinary,
    pub public_key: HexBinary,
    pub salt: HexBinary,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const UNUSED_TOKEN_ID: Item<u64> = Item::new("unused_token_id");
pub const USED_IDS: Map<String, Empty> = Map::new("used_ids");

pub const USED_CU_HASHES: Map<String, Empty> = Map::new("used_cu_hashes");
