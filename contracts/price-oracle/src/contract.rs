use crate::error::ContractError;
use crate::helpers::get_pair_id;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use crate::state::{
    CREATOR, LATEST_PRICES, LATEST_VWAP, NOD_CONTRACT_ADDRESS, PAIR_DAY_TYPES, PRICE_HISTORY,
    TOKEN_PAIRS, VWAP_CONFIG, VWAP_HISTORY,
};
use crate::types::{DayType, PriceData, TokenPair, UpdatePriceParams, VwapConfig};
use cosmwasm_schema::cw_serde;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, CosmosMsg, Decimal, DepsMut, Env, Event, MessageInfo, Response, WasmMsg,
};
use cw2::set_contract_version;
use outbe_utils::denom::Denom;

/// Message types for calling the nod contract
#[cw_serde]
pub enum NodExecuteMsg {
    PriceUpdate { price_threshold: Decimal },
}

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

    // Initialize VWAP config with default 5 minutes (300 seconds)
    let vwap_config = VwapConfig {
        window_seconds: msg.vwap_window_seconds.unwrap_or(300),
    };
    VWAP_CONFIG.save(deps.storage, &vwap_config)?;

    if let Some(nod_address) = &msg.nod_address {
        let nod_addr = deps.api.addr_validate(nod_address)?;
        NOD_CONTRACT_ADDRESS.save(deps.storage, &nod_addr)?;
    }

    Ok(Response::default()
        .add_attribute("action", "price-oracle::instantiate")
        .add_event(
            Event::new("price-oracle::instantiate")
                .add_attribute("creator", creator)
                .add_attribute(
                    "vwap_window_seconds",
                    vwap_config.window_seconds.to_string(),
                )
                .add_attribute("nod_address", msg.nod_address.unwrap_or("none".to_string())),
        ))
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
            volume,
        } => {
            let params = UpdatePriceParams {
                token1,
                token2,
                price,
                open,
                high,
                low,
                close,
                volume,
            };
            execute_update_price(deps, env, info, params)
        }
        ExecuteMsg::SetDayType {
            token1,
            token2,
            day_type,
        } => execute_set_day_type(deps, env, info, token1, token2, day_type),
        ExecuteMsg::UpdateVwapWindow { window_seconds } => {
            execute_update_vwap_window(deps, env, info, window_seconds)
        }
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
    LATEST_VWAP.remove(deps.storage, pair_id.clone());
    VWAP_HISTORY.remove(deps.storage, pair_id.clone());

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
        volume: params.volume,
    };

    // Update latest price
    LATEST_PRICES.save(deps.storage, pair_id.clone(), &price_data)?;

    // Update price history
    let mut history = PRICE_HISTORY
        .may_load(deps.storage, pair_id.clone())?
        .unwrap_or_default();
    history.push(price_data);
    PRICE_HISTORY.save(deps.storage, pair_id.clone(), &history)?;

    // TODO temporary disable updating vwap because of error
    //  rpc error: code = Unknown desc = rpc error: code = Unknown
    //  desc = failed to execute message; message index: 0: Error calling the VM:
    //  Error executing Wasm: Wasmer runtime error: RuntimeError: Generic error: Value too big.
    //  Tried to write 131112 bytes to storage, limit is 131072
    // Calculate and update VWAP
    // let vwap_config = VWAP_CONFIG.load(deps.storage)?;
    // if let Some(vwap_data) = calculate_vwap(&history, env.block.time, vwap_config.window_seconds) {
    //     // Save latest VWAP
    //     LATEST_VWAP.save(deps.storage, pair_id.clone(), &vwap_data)?;
    //
    //     // Update VWAP history
    //     let mut vwap_history = VWAP_HISTORY
    //         .may_load(deps.storage, pair_id.clone())?
    //         .unwrap_or_default();
    //     vwap_history.push(vwap_data);
    //     VWAP_HISTORY.save(deps.storage, pair_id.clone(), &vwap_history)?;
    // }

    let mut messages: Vec<CosmosMsg> = vec![];
    if pair_id == "native_coen-native_usdc" {
        if let Ok(nod_contract_addr) = NOD_CONTRACT_ADDRESS.load(deps.storage) {
            let wasm_msg = WasmMsg::Execute {
                contract_addr: nod_contract_addr.to_string(),
                msg: to_json_binary(&NodExecuteMsg::PriceUpdate {
                    price_threshold: params.price,
                })?,
                funds: vec![],
            };
            messages.push(CosmosMsg::Wasm(wasm_msg));
        }
    }

    Ok(Response::new()
        .add_attribute("action", "price-oracle::update_price")
        .add_messages(messages)
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

fn execute_update_vwap_window(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    window_seconds: u64,
) -> Result<Response, ContractError> {
    // Check authorization
    // CREATOR.assert_owner(deps.storage, &info.sender)?;

    let vwap_config = VwapConfig { window_seconds };
    VWAP_CONFIG.save(deps.storage, &vwap_config)?;

    Ok(Response::new()
        .add_attribute("action", "price-oracle::update_vwap_window")
        .add_attribute("window_seconds", window_seconds.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    match msg {
        MigrateMsg::Migrate {} => Ok(Response::new()),
    }
}
