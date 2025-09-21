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
    pub total_gratis_limit: Uint128,
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
    pub total_gratis_limit: Uint128,
    /// Total fees to be paid for validators (currently 0)
    pub total_fees: Uint128,
    /// Touch limit = 4 % of `total_emission_limit - total_fees`
    pub touch_limit: Uint128,
    /// Gold ignot price in native coins
    pub gold_ignot_price: Decimal,
}

#[cw_serde]
pub struct DailyRunState {
    pub number_of_runs: usize,
    pub last_tribute_id: Option<String>,
    pub undistributed_limit: Uint128,
}

#[cw_serde]
pub enum RunType {
    Lysis,
    Touch,
}

#[cw_serde]
pub struct RunHistoryInfo {
    /// Identifies what kind of run it was
    pub run_type: RunType,
    /// Vector rate or None for Touch
    pub vector_rate: Option<Uint128>,
    /// Lysis or Touch limit
    pub limit: Uint128,
    /// Lysis deficit or 0 for Touch
    pub deficit: Uint128,
    /// Lysis capacity or = limit for Touch
    pub capacity: Uint128,
    /// Count of tributes was assigned for this run
    pub assigned_tributes: usize,
    /// Sum of tributes were assigned for this run or touch_limit for Touch
    pub assigned_tributes_sum: Uint128,
    /// Count of winners in this run
    pub winner_tributes: usize,
    /// Winners sum in this run or touch_limit for Touch
    pub winner_tributes_sum: Uint128,
}

#[cw_serde]
pub struct DailyRunHistory {
    pub data: Vec<RunHistoryInfo>,
}
