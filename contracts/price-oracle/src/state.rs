use crate::types::{DayType, PriceData, TokenPair};
use cw_ownable::{OwnershipStore, OWNERSHIP_KEY};
use cw_storage_plus::Map;
use std::fmt;

pub const CREATOR: OwnershipStore = OwnershipStore::new(OWNERSHIP_KEY);
pub const TOKEN_PAIRS: Map<String, TokenPair> = Map::new("token_pairs");
pub const PRICE_HISTORY: Map<String, Vec<PriceData>> = Map::new("price_history");
pub const LATEST_PRICES: Map<String, PriceData> = Map::new("latest_prices");
pub const PAIR_DAY_TYPES: Map<String, DayType> = Map::new("pair_day_types");

impl fmt::Display for DayType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DayType::Green => write!(f, "GREEN"),
            DayType::Red => write!(f, "RED"),
        }
    }
}
