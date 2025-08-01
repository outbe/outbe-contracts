use crate::error::ContractError;
use crate::helpers::get_pair_id;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use crate::state::{CREATOR, LATEST_PRICES, PAIR_DAY_TYPES, PRICE_HISTORY, TOKEN_PAIRS};
use crate::types::{DayType, PriceData, TokenPair, UpdatePriceParams};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{DepsMut, Env, Event, MessageInfo, Response};
use cw2::set_contract_version;
use outbe_utils::denom::Denom;

const CONTRACT_NAME: &str = "outbe:price-oracle";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // use info.sender if None is passed
    let creator: &str = match msg.creator.as_deref() {
        Some(creator) => creator,
        None => info.sender.as_str(),
    };

    CREATOR.initialize_owner(deps.storage, deps.api, Some(creator))?;

    Ok(Response::default()
        .add_attribute("action", "price-oracle::instantiate")
        .add_event(Event::new("price-oracle::instantiate").add_attribute("creator", creator)))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddTokenPair { token1, token2 } => {
            execute_add_token_pair(deps, env, info, token1, token2)
        }
        ExecuteMsg::RemoveTokenPair { token1, token2 } => {
            execute_remove_token_pair(deps, env, info, token1, token2)
        }
        ExecuteMsg::UpdatePrice {
            token1,
            token2,
            price,
            open,
            high,
            low,
            close,
        } => {
            let params = UpdatePriceParams {
                token1,
                token2,
                price,
                open,
                high,
                low,
                close,
            };
            execute_update_price(deps, env, info, params)
        }
        ExecuteMsg::SetDayType {
            token1,
            token2,
            day_type,
        } => execute_set_day_type(deps, env, info, token1, token2, day_type),
    }
}

fn execute_add_token_pair(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    token1: Denom,
    token2: Denom,
) -> Result<Response, ContractError> {
    // Check authorization
    // CREATOR.assert_owner(deps.storage, &info.sender)?;

    // Validate tokens are different
    if token1 == token2 {
        return Err(ContractError::InvalidTokenPair {});
    }

    let pair_id = get_pair_id(&token1, &token2);

    // Check if pair already exists
    if TOKEN_PAIRS.has(deps.storage, pair_id.clone()) {
        return Err(ContractError::PairAlreadyExists { pair_id });
    }

    // Save token pair
    TOKEN_PAIRS.save(
        deps.storage,
        pair_id.clone(),
        &TokenPair {
            token1: token1.clone(),
            token2: token2.clone(),
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "price-oracle::add_token_pair")
        .add_event(
            Event::new("price-oracle::pair_added")
                .add_attribute("pair_id", pair_id)
                .add_attribute("token1", token1.to_string())
                .add_attribute("token2", token2.to_string()),
        ))
}

fn execute_remove_token_pair(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    token1: Denom,
    token2: Denom,
) -> Result<Response, ContractError> {
    // Check authorization
    // CREATOR.assert_owner(deps.storage, &info.sender)?;

    // Validate tokens are different
    if token1 == token2 {
        return Err(ContractError::InvalidTokenPair {});
    }

    let pair_id = get_pair_id(&token1, &token2);

    // Check if pair exists
    if !TOKEN_PAIRS.has(deps.storage, pair_id.clone()) {
        return Err(ContractError::PairNotFound { pair_id });
    }

    // Remove token pair and associated data
    TOKEN_PAIRS.remove(deps.storage, pair_id.clone());
    LATEST_PRICES.remove(deps.storage, pair_id.clone());
    PRICE_HISTORY.remove(deps.storage, pair_id.clone());
    PAIR_DAY_TYPES.remove(deps.storage, pair_id.clone());

    Ok(Response::new()
        .add_attribute("action", "price-oracle::remove_token_pair")
        .add_event(Event::new("price-oracle::pair_removed").add_attribute("pair_id", pair_id)))
}

fn execute_update_price(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    params: UpdatePriceParams,
) -> Result<Response, ContractError> {
    // Check authorization
    // CREATOR.assert_owner(deps.storage, &info.sender)?;

    // Validate tokens are different
    if params.token1 == params.token2 {
        return Err(ContractError::InvalidTokenPair {});
    }

    let pair_id = get_pair_id(&params.token1, &params.token2);

    // Check if pair exists
    if !TOKEN_PAIRS.has(deps.storage, pair_id.clone()) {
        return Err(ContractError::PairNotFound { pair_id });
    }

    let price_data = PriceData {
        price: params.price,
        timestamp: env.block.time,
        open: params.open,
        high: params.high,
        low: params.low,
        close: params.close,
    };

    // Update latest price
    LATEST_PRICES.save(deps.storage, pair_id.clone(), &price_data)?;

    // Update price history
    let mut history = PRICE_HISTORY
        .may_load(deps.storage, pair_id.clone())?
        .unwrap_or_default();
    history.push(price_data);
    PRICE_HISTORY.save(deps.storage, pair_id.clone(), &history)?;

    Ok(Response::new()
        .add_attribute("action", "price-oracle::update_price")
        .add_event(
            Event::new("price-oracle::price_updated")
                .add_attribute("pair_id", pair_id)
                .add_attribute("price", params.price.to_string())
                .add_attribute("timestamp", env.block.time.seconds().to_string())
                .add_attribute("updated_by", info.sender),
        ))
}

fn execute_set_day_type(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token1: Denom,
    token2: Denom,
    day_type: DayType,
) -> Result<Response, ContractError> {
    // Check authorization
    // CREATOR.assert_owner(deps.storage, &info.sender)?;

    // Validate tokens are different
    if token1 == token2 {
        return Err(ContractError::InvalidTokenPair {});
    }

    let pair_id = get_pair_id(&token1, &token2);

    // Check if pair exists
    if !TOKEN_PAIRS.has(deps.storage, pair_id.clone()) {
        return Err(ContractError::PairNotFound { pair_id });
    }

    // Save day type
    PAIR_DAY_TYPES.save(deps.storage, pair_id.clone(), &day_type)?;

    Ok(Response::new()
        .add_attribute("action", "price-oracle::set_day_type")
        .add_event(
            Event::new("price-oracle::day_type_set")
                .add_attribute("pair_id", pair_id)
                .add_attribute("token1", token1.to_string())
                .add_attribute("token2", token2.to_string())
                .add_attribute("day_type", day_type.to_string())
                .add_attribute("updated_by", info.sender),
        ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    match msg {
        MigrateMsg::Migrate {} => Ok(Response::new()),
    }
}
