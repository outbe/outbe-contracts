use crate::state::{CREATOR, TOKEN_PAIR_PRICE};
use crate::types::TokenPairPrice;
use cosmwasm_schema::{cw_serde, QueryResponses};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Addr, Binary, Deps, Env, StdResult, Storage};
use cw_ownable::Ownership;

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(TokenPairPrice)]
    GetPrice {},
    #[returns(cw_ownable::Ownership<String>)]
    GetCreatorOwnership {},
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPrice {} => to_json_binary(&query_price(deps.storage)?),
        QueryMsg::GetCreatorOwnership {} => to_json_binary(&query_creator_ownership(deps.storage)?),
    }
}

fn query_price(storage: &dyn Storage) -> StdResult<TokenPairPrice> {
    let price_data = TOKEN_PAIR_PRICE.load(storage)?;
    Ok(TokenPairPrice {
        token1: price_data.token1,
        token2: price_data.token2,
        day_type: price_data.day_type,
        price: price_data.price,
    })
}

pub fn query_creator_ownership(storage: &dyn Storage) -> StdResult<Ownership<Addr>> {
    CREATOR.get_ownership(storage)
}
