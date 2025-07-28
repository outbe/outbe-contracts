use crate::state::{DailyRunInfo, DAILY_RUNS, DAILY_RUNS_INFO, TRIBUTES_DISTRIBUTION};
use cosmwasm_schema::{cw_serde, QueryResponses};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, Order, StdResult, Uint128};

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(DailyRunsResponse)]
    DailyRuns {},
    #[returns(TributesDistributionResponse)]
    TributesDistribution {},
}

#[cw_serde]
pub struct DailyRunsData {
    /// timestamp of the date when raffles were made (seconds)
    pub timestamp: u64,
    /// counter of the raffles in that day
    pub runs_happened: usize,
    pub info: DailyRunInfo,
}
#[cw_serde]
pub struct DailyRunsResponse {
    pub data: Vec<DailyRunsData>,
}

#[cw_serde]
pub struct AllocationResponse {
    pub total_allocation: Uint128,
    pub pool_allocation: Uint128,
}

#[cw_serde]
pub struct TributesDistributionData {
    /// the key is in format `{DATE_TIMESTAMP}_{RAFFLE_RUN_ID}_{TRIBUTE_INDEX}` for emulate buckets
    /// where `DATE_TIMESTAMP` is the metadosis date
    /// `RAFFLE_RUN_ID` is in range [1..24]
    /// `TRIBUTE_INDEX` starts from 0, unique withing the bucket
    pub key: String,
    /// Tribute identifier
    pub tribute_id: String,
}

#[cw_serde]
pub struct TributesDistributionResponse {
    pub data: Vec<TributesDistributionData>,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::DailyRuns {} => to_json_binary(&query_daily_runs(deps, env)?),
        QueryMsg::TributesDistribution {} => {
            to_json_binary(&query_tributes_distribution(deps, env)?)
        }
    }
}

fn query_daily_runs(deps: Deps, _env: Env) -> StdResult<DailyRunsResponse> {
    let result: StdResult<Vec<DailyRunsData>> = DAILY_RUNS_INFO
        .range(deps.storage, None, None, Order::Ascending)
        .filter_map(|item| match item {
            Ok((k, v)) => {
                let runs_happened = DAILY_RUNS.load(deps.storage, k).unwrap_or(0);

                Some(Ok(DailyRunsData {
                    timestamp: k,
                    info: v,
                    runs_happened,
                }))
            }
            _ => None,
        })
        .collect();

    Ok(DailyRunsResponse { data: result? })
}

fn query_tributes_distribution(deps: Deps, _env: Env) -> StdResult<TributesDistributionResponse> {
    println!("query_tributes_distribution");
    let result: StdResult<Vec<TributesDistributionData>> = TRIBUTES_DISTRIBUTION
        .range(deps.storage, None, None, Order::Ascending)
        .filter_map(|item| match item {
            Ok((k, v)) => {
                println!("found tribute {} {}", k, v);
                Some(Ok(TributesDistributionData {
                    key: k,
                    tribute_id: v,
                }))
            }
            e => {
                println!("debug error {:?}", e);
                None
            }
        })
        .collect();

    Ok(TributesDistributionResponse { data: result? })
}
