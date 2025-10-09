use crate::state::{
    Config, DailyRunState, Entry, MetadosisInfo, CONFIG, DAILY_RUN_STATE, ENTRY_STATE,
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
    #[returns(ConfigResponse)]
    Config {},
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
    pub data: Vec<Entry>,
}

#[cw_serde]
pub struct ConfigResponse {
    pub data: Config,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::MetadosisInfo {} => to_json_binary(&query_metadosis_info(deps, env)?),
        QueryMsg::History {} => to_json_binary(&query_history(deps, env)?),
        QueryMsg::Config {} => to_json_binary(&query_config(deps, env)?),
    }
}

fn query_metadosis_info(deps: Deps, _env: Env) -> StdResult<MetadosisInfoResponse> {
    let result: StdResult<Vec<MetadosisInfoData>> = METADOSIS_INFO
        .range(deps.storage, None, None, Order::Ascending)
        .filter_map(|item| match item {
            Ok((k, v)) => {
                let state = DAILY_RUN_STATE
                    .load(deps.storage, k)
                    .unwrap_or(DailyRunState { number_of_runs: 0 });

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
    let result: StdResult<Vec<Entry>> = ENTRY_STATE
        .range(deps.storage, None, None, Order::Ascending)
        .filter_map(|item| match item {
            Ok((_, v)) => Some(Ok(v)),
            _ => None,
        })
        .collect();

    Ok(HistoryResponse { data: result? })
}

fn query_config(deps: Deps, _env: Env) -> StdResult<ConfigResponse> {
    let result = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { data: result })
}
