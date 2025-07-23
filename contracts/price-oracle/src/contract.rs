use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use crate::state::{TokenPairState, CREATOR, TOKEN_PAIR_PRICE};
use crate::types::TokenPairPrice;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{DepsMut, Env, Event, MessageInfo, Response};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "outbe:price-oracle";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
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

    let token_pair = TokenPairState {
        token1: msg.initial_price.token1,
        token2: msg.initial_price.token2,
        price: msg.initial_price.price,
        day_type: msg.initial_price.day_type,
        last_updated: env.block.time,
    };
    TOKEN_PAIR_PRICE.save(deps.storage, &token_pair)?;

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
        ExecuteMsg::UpdatePrice { token_pair_price } => {
            execute_update_price(deps, env, info, token_pair_price)
        }
    }
}

fn execute_update_price(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_pair_price: TokenPairPrice,
) -> Result<Response, ContractError> {
    let token1_str = token_pair_price.token1.to_string();
    let token2_str = token_pair_price.token2.to_string();

    let token_pair = TokenPairState {
        token1: token_pair_price.token1,
        token2: token_pair_price.token2,
        price: token_pair_price.price,
        day_type: token_pair_price.day_type.clone(),
        last_updated: env.block.time,
    };

    TOKEN_PAIR_PRICE.save(deps.storage, &token_pair)?;

    Ok(Response::new()
        .add_attribute("action", "price-oracle::execute_update_price")
        .add_event(
            Event::new("price-oracle::price_updated")
                .add_attribute("token1", token1_str)
                .add_attribute("token2", token2_str)
                .add_attribute("price", token_pair_price.price.to_string())
                .add_attribute("day_type", token_pair_price.day_type.to_string())
                .add_attribute("timestamp", env.block.time.seconds().to_string())
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
