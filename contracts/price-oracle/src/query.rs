use crate::state::{CREATOR, TOKEN_PAIR_PRICE};
use crate::types::TokenPairPrice;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{to_json_binary, Addr, Binary, Deps, Env, StdError, StdResult, Storage};
use cw20::Denom;
use cw_ownable::Ownership;

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(TokenPairPrice)]
    GetPrice {},
    #[returns(TokenPairPrice)]
    GetPriceByTokenPair {
        token1: cw20::Denom,
        token2: cw20::Denom,
    },
    #[returns(cw_ownable::Ownership<String>)]
    GetCreatorOwnership {},
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPrice {} => to_json_binary(&query_price(deps.storage)?),
        QueryMsg::GetPriceByTokenPair { token1, token2 } => {
            to_json_binary(&query_price_by_token_pair(deps.storage, token1, token2)?)
        }
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

fn query_price_by_token_pair(
    storage: &dyn Storage,
    token1: Denom,
    token2: Denom,
) -> StdResult<TokenPairPrice> {
    let price_data = TOKEN_PAIR_PRICE.load(storage)?;

    if (price_data.token1 == token1 && price_data.token2 == token2)
        || (price_data.token1 == token2 && price_data.token2 == token1)
    {
        let price = if price_data.token1 == token1 && price_data.token2 == token2 {
            price_data.price
        } else {
            cosmwasm_std::Decimal::one()
                .checked_div(price_data.price)
                .unwrap_or(cosmwasm_std::Decimal::zero())
        };

        Ok(TokenPairPrice {
            token1,
            token2,
            day_type: price_data.day_type,
            price,
        })
    } else {
        Err(StdError::not_found("Token pair not found"))
    }
}

pub fn query_creator_ownership(storage: &dyn Storage) -> StdResult<Ownership<Addr>> {
    CREATOR.get_ownership(storage)
}
