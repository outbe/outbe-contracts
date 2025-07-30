use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_ownable::{OwnershipStore, OWNERSHIP_KEY};
use cw_storage_plus::{Item, Map};
use outbe_utils::date::WorldwideDay;

#[cw_serde]
pub struct Config {
    pub vector: Option<Addr>,
    pub tribute: Option<Addr>,
    pub nod: Option<Addr>,
    pub token_allocator: Option<Addr>,
    pub price_oracle: Option<Addr>,
    pub random_oracle: Option<Addr>,
    /// Percentage of Total Tribute Interest
    pub deficit: Decimal,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const CREATOR: OwnershipStore = OwnershipStore::new(OWNERSHIP_KEY);

/// Map containing data for run metadosis for each dey. Filled during `Prepare` phase.
pub const METADOSIS_INFO: Map<WorldwideDay, MetadosisInfo> = Map::new("metadosis_info");

/// Map to track how many runs were happened for each day
pub const DAILY_RUN_STATE: Map<WorldwideDay, DailyRunState> = Map::new("daily_runs");

/// Saves history to show on UI
pub const DAILY_RUNS_HISTORY: Map<WorldwideDay, DailyRunHistory> = Map::new("daily_runs_history");

/// Saves winners to do not peek them in Touch
pub const WINNERS: Map<String, ()> = Map::new("tribute_winners");

#[cw_serde]
pub enum MetadosisInfo {
    LysisAndTouch {
        lysis_info: LysisInfo,
        touch_info: TouchInfo,
    },
    Touch {
        touch_info: TouchInfo,
    },
}

#[cw_serde]
pub struct LysisInfo {
    /// Total emission limit in native coins for this day
    pub total_emission_limit: Uint128,
    /// Total fees to be paid for validators (currently 0)
    pub total_fees: Uint128,
    /// Total Lysis Limit = `total_emission_limit - total_fees`
    pub total_lysis_limit: Uint128,
    /// Lysis limit = `total_lysis_limit / 24`
    pub lysis_limit: Uint128,
    /// Total Tributes interest
    pub total_tribute_interest: Uint128,
    /// Total Deficit
    pub total_deficit: Uint128,
    /// Deficits for each execution where the index in 0..23 corresponds for each daily execution
    pub lysis_deficits: Vec<Uint128>,
    /// Vector rates for each execution where the index in 0..23 corresponds for each daily execution
    pub vector_rates: Vec<Uint128>,
}

#[cw_serde]
pub struct TouchInfo {
    /// Total emission limit in native coins for this day
    pub total_emission_limit: Uint128,
    /// Total fees to be paid for validators (currently 0)
    pub total_fees: Uint128,
    /// Touch limit = `(total_emission_limit - total_fees) / 24`
    pub touch_limit: Uint128,
    /// Gold ignot price in native coins
    pub gold_ignot_price: Decimal,
}

#[cw_serde]
pub struct DailyRunState {
    pub number_of_runs: usize,
    pub last_tribute_id: Option<String>,
}

#[cw_serde]
pub enum RunType {
    Lysis,
    Touch,
}

#[cw_serde]
pub struct RunHistoryInfo {
    pub run_type: RunType,
    pub vector_rate: Option<Decimal>,
    pub pool_allocation: Uint128,
    pub pool_deficit: Uint128,
    pub pool_capacity: Uint128,
    pub assigned_tributes: usize,
    pub assigned_tributes_sum: Uint128,
    pub winner_tributes: usize,
    pub winner_tributes_sum: Uint128,
}

#[cw_serde]
pub struct DailyRunHistory {
    pub data: Vec<RunHistoryInfo>,
}
