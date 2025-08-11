use crate::types::{DayType, PriceData, TokenPair, VwapConfig, VwapData};
use cw_ownable::{OwnershipStore, OWNERSHIP_KEY};
use cw_storage_plus::{Item, Map};
use std::fmt;

pub const CREATOR: OwnershipStore = OwnershipStore::new(OWNERSHIP_KEY);
pub const TOKEN_PAIRS: Map<String, TokenPair> = Map::new("token_pairs");
pub const PRICE_HISTORY: Map<String, Vec<PriceData>> = Map::new("price_history");
pub const LATEST_PRICES: Map<String, PriceData> = Map::new("latest_prices");
pub const PAIR_DAY_TYPES: Map<String, DayType> = Map::new("pair_day_types");
pub const VWAP_CONFIG: Item<VwapConfig> = Item::new("vwap_config");
pub const LATEST_VWAP: Map<String, VwapData> = Map::new("latest_vwap");
pub const VWAP_HISTORY: Map<String, Vec<VwapData>> = Map::new("vwap_history");

impl fmt::Display for DayType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DayType::Green => write!(f, "GREEN"),
            DayType::Red => write!(f, "RED"),
        }
    }
}
