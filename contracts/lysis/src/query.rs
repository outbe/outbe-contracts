use crate::state::TRIBUTES_DISTRIBUTION;
use cosmwasm_schema::{cw_serde, QueryResponses};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, Order, StdResult, Timestamp};

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // #[returns(DailyRaffleResponse)]
    // DailyRaffle {},
    #[returns(TributesDistributionResponse)]
    TributesDistribution {},
}

#[cw_serde]
pub struct DailyRaffleData {
    /// timestamp of the date when raffles were made
    pub timestamp: Timestamp,
    /// counter of the raffles in that day
    pub runs: u16,
}
#[cw_serde]
pub struct DailyRaffleResponse {
    pub data: Vec<DailyRaffleData>,
}

#[cw_serde]
pub struct TributesDistributionData {
    /// the key is in format `{DATE_TIMESTAMP}_{RAFFLE_RUN_ID}_{TRIBUTE_INDEX}` for emulate buckets
    /// where `DATE_TIMESTAMP` is the lysis date
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
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        // QueryMsg::DailyRaffle {} => to_json_binary(&query_daily_raffle(_deps, _env)?),
        QueryMsg::TributesDistribution {} => {
            to_json_binary(&query_tributes_distribution(_deps, _env)?)
        }
    }
}

// fn query_daily_raffle(deps: Deps, _env: Env) -> StdResult<DailyRaffleResponse> {
//     let result: StdResult<Vec<DailyRaffleData>> = DAILY_RAFFLE
//         .range(deps.storage, None, None, Order::Ascending)
//         .filter_map(|item| match item {
//             Ok((k, v)) => Some(Ok(DailyRaffleData {
//                 timestamp: Timestamp::from_seconds(k),
//                 runs: v,
//             })),
//             _ => None,
//         })
//         .collect();
//
//     Ok(DailyRaffleResponse { data: result? })
// }
//
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

// fn query_history(deps: Deps, _env: Env) -> StdResult<RaffleHistory> {
//     let history = HISTORY
//         .may_load(deps.storage)?
//         .unwrap_or(RaffleHistory { data: vec![] });
//
//     Ok(history)
// }
