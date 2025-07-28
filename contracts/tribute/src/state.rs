use crate::types::{TributeConfig, TributeNft};
use cosmwasm_std::{StdError, StdResult, Storage};
use cw_storage_plus::{Item, Map};
use outbe_nft::state::CollectionInfo;

pub struct TributeState {
    /// Dynamic map where Map's namespace is a tribute day so all tributes from a single day
    /// are placed into the single map.
    /// TODO use indexed map to make index by owner
    pub daily_tributes: Map<String, TributeNft>,
    /// Collection Info
    pub collection_info: Item<CollectionInfo>,
    /// Collection Config
    pub collection_config: Item<TributeConfig>,
    /// Count of all Tributes at the moment
    pub token_count: Item<u64>,
    /// Active Days holds all day strings that can be used as namespace keys for `daily_tributes`
    pub active_days: Item<Vec<String>>,
}

pub const DEFAULT_DATE: &str = "2000-01-01";

impl TributeState {
    pub fn for_day(day: String) -> Self {
        TributeState {
            daily_tributes: Map::new_dyn(format!("daily_tributes_{}", day.replace("-", ""))),
            collection_info: Item::new("collection_info"),
            collection_config: Item::new("collection_config"),
            token_count: Item::new("token_count"),
            active_days: Item::new("active_days"),
        }
    }

    pub fn default() -> Self {
        Self::for_day(DEFAULT_DATE.to_string())
    }

    pub fn init(
        storage: &mut dyn Storage,
        info: &CollectionInfo,
        config: &TributeConfig,
    ) -> StdResult<Self> {
        let state = Self::default();
        state.active_days.update(storage, |mut active_days| {
            if !active_days.contains(&DEFAULT_DATE.to_string()) {
                active_days.push(DEFAULT_DATE.to_string());
            }
            Ok::<Vec<String>, StdError>(active_days)
        })?;
        state.collection_config.save(storage, config)?;
        state.collection_info.save(storage, info)?;
        state.token_count.save(storage, &0)?;

        Ok(state)
    }
}
