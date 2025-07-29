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

#[cw_serde]
pub enum RunType {
    Lysis,
    Touch,
}

#[cw_serde]
pub struct RunInfo {
    pub vector_index: u16,
    pub run_type: RunType,

    pub total_allocation: Uint128,
    pub pool_allocation: Uint128,
    pub total_deficit: Uint128,
    pub pool_deficit: Uint128,
    pub pool_capacity: Uint128,
    pub assigned_tributes: usize,
    pub assigned_tributes_sum: Uint128,
}

#[cw_serde]
pub struct DailyRunInfo {
    pub data: Vec<RunInfo>,
    pub number_of_runs: usize,
}

pub const DAILY_RUNS: Map<WorldwideDay, usize> = Map::new("daily_runs");
pub const DAILY_RUNS_INFO: Map<WorldwideDay, DailyRunInfo> = Map::new("daily_runs_info");

pub const TRIBUTES_DISTRIBUTION: Map<&str, String> = Map::new("tributes_distribution");

#[cw_serde]
pub enum MetadosisInfo {
    LysisAndTouch {
        /// Total emission limit in native coins for this day
        total_emission_limit: Uint128,
        /// Total fees to be paid for validators (currently 0)
        total_fees: Uint128,
        /// Total Lysis Limit = `total_emission_limit - total_fees`
        total_lysis_limit: Uint128,
        /// Lysis limit = `total_lysis_limit / 24`
        lysis_limit: Uint128,
        /// Total Tributes interest
        total_tribute_interest: Uint128,
        /// Total Deficit
        total_deficit: Uint128,
        /// Deficits for each execution where the index in 0..23 corresponds for each daily execution
        lysis_deficits: Vec<Uint128>,
        /// Vector rates for each execution where the index in 0..23 corresponds for each daily execution
        vector_rates: Vec<Uint128>,

        /// Gold ignot price in native coins
        gold_ignot_price: Decimal,
    },
    Touch {
        /// Total emission limit in native coins for this day
        total_emission_limit: Uint128,
        /// Total fees to be paid for validators (currently 0)
        total_fees: Uint128,
        /// Lysis limit = `(total_emission_limit - total_fees) / 24`
        touch_limit: Uint128,
        /// Gold ignot price in native coins
        gold_ignot_price: Decimal,
    },
}

/// Map containing data for run metadosis for each dey. Filled during `Prepare` phase.
pub const METADOSIS_INFO: Map<WorldwideDay, MetadosisInfo> = Map::new("metadosis_info");
