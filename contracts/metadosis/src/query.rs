use crate::state::{
    DailyRunHistory, DailyRunState, MetadosisInfo, DAILY_RUNS_HISTORY, DAILY_RUN_STATE,
    METADOSIS_INFO,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, Order, StdResult};
use outbe_utils::date::WorldwideDay;

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(MetadosisInfoResponse)]
    MetadosisInfo {},
    #[returns(HistoryResponse)]
    History {},
}

#[cw_serde]
pub struct MetadosisInfoData {
    pub date: WorldwideDay,
    pub info: MetadosisInfo,
    pub state: DailyRunState,
}

#[cw_serde]
pub struct MetadosisInfoResponse {
    pub data: Vec<MetadosisInfoData>,
}

#[cw_serde]
pub struct HistoryResponse {
    pub data: Vec<HistoryData>,
}

#[cw_serde]
pub struct HistoryData {
    pub date: WorldwideDay,
    pub daily_run_history: DailyRunHistory,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::MetadosisInfo {} => to_json_binary(&query_metadosis_info(deps, env)?),
        QueryMsg::History {} => to_json_binary(&query_history(deps, env)?),
    }
}

fn query_metadosis_info(deps: Deps, _env: Env) -> StdResult<MetadosisInfoResponse> {
    let result: StdResult<Vec<MetadosisInfoData>> = METADOSIS_INFO
        .range(deps.storage, None, None, Order::Ascending)
        .filter_map(|item| match item {
            Ok((k, v)) => {
                let state = DAILY_RUN_STATE
                    .load(deps.storage, k)
                    .unwrap_or(DailyRunState {
                        last_tribute_id: None,
                        number_of_runs: 0,
                    });

                Some(Ok(MetadosisInfoData {
                    date: k,
                    info: v,
                    state,
                }))
            }
            _ => None,
        })
        .collect();

    Ok(MetadosisInfoResponse { data: result? })
}

fn query_history(deps: Deps, _env: Env) -> StdResult<HistoryResponse> {
    let result: StdResult<Vec<HistoryData>> = DAILY_RUNS_HISTORY
        .range(deps.storage, None, None, Order::Ascending)
        .filter_map(|item| match item {
            Ok((k, v)) => Some(Ok(HistoryData {
                date: k,
                daily_run_history: v,
            })),
            _ => None,
        })
        .collect();

    Ok(HistoryResponse { data: result? })
}
