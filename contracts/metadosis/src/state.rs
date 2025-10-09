use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Timestamp, Uint128};
use cw_ownable::{OwnershipStore, OWNERSHIP_KEY};
use cw_storage_plus::{Item, Map};
use outbe_utils::date::WorldwideDay;

#[cw_serde]
pub struct Config {
    pub tribute: Option<Addr>,
    pub nod: Option<Addr>,
    pub token_allocator: Option<Addr>,
    pub price_oracle: Option<Addr>,
    pub random_oracle: Option<Addr>,
    /// Percentage of Total Tribute Interest
    pub lysis_limit_percent: Decimal,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const CREATOR: OwnershipStore = OwnershipStore::new(OWNERSHIP_KEY);

/// Map containing data for run metadosis for each dey. Filled during `Prepare` phase.
pub const METADOSIS_INFO: Map<WorldwideDay, MetadosisInfo> = Map::new("metadosis_info");

/// Map to track how many runs were happened for each day
pub const DAILY_RUN_STATE: Map<WorldwideDay, DailyRunState> = Map::new("daily_runs");

/// Saves winners to do not peek them in Touch
pub const WINNERS: Map<String, ()> = Map::new("tribute_winners");

#[cw_serde]
pub enum MetadosisInfo {
    Lysis { lysis_info: LysisInfo },
    Touch { touch_info: TouchInfo },
}

#[cw_serde]
pub struct LysisInfo {
    /// Total emission limit in native coins for this day
    pub total_gratis_limit: Uint128,
    /// Total fees to be paid for validators (currently 0)
    pub total_fees: Uint128,
    /// Total Lysis Limit = `total_emission_limit - total_fees`
    pub total_lysis_limit: Uint128,
    /// Total Tributes interest
    pub total_tribute_interest: Uint128,
    /// Total Deficit
    pub total_lysis_deficit: Uint128,
    pub distribution_percent: Decimal,
}

#[cw_serde]
pub struct TouchInfo {
    /// Total emission limit in native coins for this day
    pub total_gratis_limit: Uint128,
    /// Touch limit = 4 % of `total_emission_limit - total_fees`
    pub touch_limit: Uint128,
    /// Gold ignot price in native coins
    pub gold_ignot_price: Decimal,
}

#[cw_serde]
pub struct DailyRunState {
    pub number_of_runs: usize,
}

#[cw_serde]
pub struct LysisEntity {
    /// Lysis ID
    pub id: String,
    /// Position in the daily lysis sequence
    pub index: usize,
    /// Lysis limit
    pub limit: Uint128,
    /// Lysis deficit
    pub deficit: Uint128,
    pub total_tribute_interest: Uint128,
    /// Worldwide day
    pub worldwide_day: WorldwideDay,
    /// Timestamp of the last tribute was recognized
    pub timestamp: Timestamp,
    /// Total emission limit in native coins for this day
    pub total_gratis_limit: Uint128,
    /// Count of tributes was assigned for this run
    pub assigned_tributes: usize,
    /// Sum of tributes were assigned for this run
    pub assigned_tributes_sum: Uint128,
}

#[cw_serde]
pub struct TouchEntity {
    /// Touch ID
    pub id: String,
    /// Worldwide day
    pub worldwide_day: WorldwideDay,
    /// Total emission limit in native coins for this day
    pub total_gratis_limit: Uint128,
    /// Gold ignot price in native coins
    pub gold_ignot_price: Decimal,
    /// Touch limit
    pub touch_limit: Uint128,
    /// Count of tributes was assigned for this run
    pub assigned_tributes: usize,
    /// IDs of the recognized tributes
    pub recognised_tributes: Vec<String>,
    /// Timestamp of the last tribute was recognized
    pub timestamp: Timestamp,
}

#[cw_serde]
pub enum Entry {
    Lysis(LysisEntity),
    Touch(TouchEntity),
}

pub const ENTRY_STATE: Map<WorldwideDay, Entry> = Map::new("entry_state");
