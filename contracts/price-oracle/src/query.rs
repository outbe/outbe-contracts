use crate::error::ContractError;
use crate::helpers::get_pair_id;

use crate::state::{CREATOR, LATEST_PRICES, PAIR_DAY_TYPES, PRICE_HISTORY, TOKEN_PAIRS};
use crate::types::{DayType, PriceData, TokenPair, TokenPairPrice};
use cosmwasm_schema::{cw_serde, QueryResponses};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, Env, Order, StdError, StdResult, Storage, Timestamp,
};
use cw_ownable::Ownership;
use outbe_utils::denom::Denom;

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // Legacy queries
    #[returns(TokenPairPrice)]
    GetPrice {},
    #[returns(cw_ownable::Ownership<String>)]
    GetCreatorOwnership {},

    // New queries
    #[returns(PriceData)]
    GetLatestPrice { token1: Denom, token2: Denom },
    #[returns(Vec<PriceData>)]
    GetPriceHistory {
        token1: Denom,
        token2: Denom,
        start_time: Timestamp,
        end_time: Timestamp,
    },
    #[returns(Vec<TokenPair>)]
    GetAllPairs {},
    #[returns(DayType)]
    GetDayType { token1: Denom, token2: Denom },
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPrice {} => to_json_binary(&query_price(deps.storage)?),
        QueryMsg::GetCreatorOwnership {} => to_json_binary(&query_creator_ownership(deps.storage)?),
        QueryMsg::GetLatestPrice { token1, token2 } => to_json_binary(
            &query_latest_price(deps.storage, token1, token2)
                .map_err(|e| StdError::generic_err(e.to_string()))?,
        ),
        QueryMsg::GetPriceHistory {
            token1,
            token2,
            start_time,
            end_time,
        } => to_json_binary(
            &query_price_history(deps.storage, token1, token2, start_time, end_time)
                .map_err(|e| StdError::generic_err(e.to_string()))?,
        ),
        QueryMsg::GetAllPairs {} => to_json_binary(&query_all_pairs(deps.storage)?),
        QueryMsg::GetDayType { token1, token2 } => to_json_binary(
            &query_day_type(deps.storage, token1, token2)
                .map_err(|e| StdError::generic_err(e.to_string()))?,
        ),
    }
}

fn query_price(storage: &dyn Storage) -> StdResult<TokenPairPrice> {
    let token1 = Denom::Native("coen".to_string());
    let token2 = Denom::Native("usdc".to_string());

    let price_data = query_latest_price(storage,token1.clone(),token2.clone()).unwrap();
    let day_type = query_day_type(storage, token1.clone(), token2.clone()).unwrap();

    Ok(TokenPairPrice {
        token1,
        token2,
        day_type,
        price: price_data.price,
    })
}

pub fn query_creator_ownership(storage: &dyn Storage) -> StdResult<Ownership<Addr>> {
    CREATOR.get_ownership(storage)
}

fn query_latest_price(
    storage: &dyn Storage,
    token1: Denom,
    token2: Denom,
) -> Result<PriceData, ContractError> {
    // Validate tokens are different
    if token1 == token2 {
        return Err(ContractError::InvalidTokenPair {});
    }

    let pair_id = get_pair_id(&token1, &token2);

    LATEST_PRICES
        .may_load(storage, pair_id.clone())
        .map_err(ContractError::Std)?
        .ok_or_else(|| ContractError::LatestPriceNotFound { pair_id })
}

fn query_price_history(
    storage: &dyn Storage,
    token1: Denom,
    token2: Denom,
    start_time: Timestamp,
    end_time: Timestamp,
) -> Result<Vec<PriceData>, ContractError> {
    // Validate tokens are different
    if token1 == token2 {
        return Err(ContractError::InvalidTokenPair {});
    }

    let pair_id = get_pair_id(&token1, &token2);

    // Validate time range
    if start_time >= end_time {
        return Err(ContractError::InvalidTimeRange {});
    }

    let history = PRICE_HISTORY
        .may_load(storage, pair_id.clone())?
        .unwrap_or_default();

    // Filter by time range
    let filtered_history: Vec<PriceData> = history
        .into_iter()
        .filter(|price_data| price_data.timestamp >= start_time && price_data.timestamp <= end_time)
        .collect();

    Ok(filtered_history)
}

fn query_all_pairs(storage: &dyn Storage) -> StdResult<Vec<TokenPair>> {
    let pairs: StdResult<Vec<TokenPair>> = TOKEN_PAIRS
        .range(storage, None, None, Order::Ascending)
        .map(|item| item.map(|(_, pair)| pair))
        .collect();

    pairs
}

fn query_day_type(
    storage: &dyn Storage,
    token1: Denom,
    token2: Denom,
) -> Result<DayType, ContractError> {
    // Validate tokens are different
    if token1 == token2 {
        return Err(ContractError::InvalidTokenPair {});
    }

    let pair_id = get_pair_id(&token1, &token2);

    PAIR_DAY_TYPES
        .may_load(storage, pair_id.clone())
        .map_err(ContractError::Std)?
        .ok_or_else(|| ContractError::DayTypeNotFound { pair_id })
}
