use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, SubmitExtension};
use crate::types::{NodConfig, NodData, NodNft, State};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Decimal, DepsMut, Env, Event, MessageInfo, Response};
use cw_ownable::OwnershipStore;
use outbe_nft::error::Cw721ContractError;
use outbe_nft::state::{CollectionInfo, Cw721Config, NftInfo};

const CONTRACT_NAME: &str = "outbe.net:nod";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Price updater owns the ability to call PriceUpdate
pub const PRICE_UPDATER: OwnershipStore = OwnershipStore::new("price_updater");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let cfg = NodConfig {};
    let collection_info = CollectionInfo {
        name: msg.name,
        symbol: msg.symbol,
        updated_at: env.block.time,
    };
    let config = Cw721Config::<NodData, NodConfig>::default();
    config.collection_config.save(deps.storage, &cfg)?;
    config
        .collection_info
        .save(deps.storage, &collection_info)?;

    let minter = msg
        .minter
        .clone()
        .unwrap_or_else(|| info.sender.to_string());
    outbe_nft::execute::initialize_minter(deps.storage, deps.api, Some(&minter))?;

    let creator = msg
        .creator
        .clone()
        .unwrap_or_else(|| info.sender.to_string());
    outbe_nft::execute::initialize_creator(deps.storage, deps.api, Some(&creator))?;

    Ok(Response::new()
        .add_attribute("action", "nod::instantiate")
        .add_attribute("minter", minter)
        .add_attribute("creator", creator))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Submit {
            token_id,
            owner,
            extension,
        } => execute_submit(deps, &env, &info, token_id, owner, *extension),
        ExecuteMsg::Burn { token_id } => execute_burn(deps, &env, &info, token_id),
        ExecuteMsg::PriceUpdate { price_threshold } => {
            execute_update_tokens_to_qualified(deps, &env, &info, price_threshold)
        }
        ExecuteMsg::UpdatePriceUpdater { price_updater } => {
            execute_update_price_updater(deps, &env, &info, price_updater)
        }
        #[cfg(feature = "demo")]
        ExecuteMsg::BurnAll { batch_size } => execute_burn_all(deps, &env, &info, batch_size),
    }
}

#[allow(clippy::too_many_arguments)]
fn execute_submit(
    deps: DepsMut,
    env: &Env,
    _info: &MessageInfo,
    token_id: String,
    owner: String,
    extension: SubmitExtension,
) -> Result<Response, ContractError> {
    // TODO uncomment after demo
    // outbe_nft::execute::assert_minter(deps.storage, &info.sender)?;

    let owner_addr = deps.api.addr_validate(&owner)?;
    let entity = extension.entity;

    let node_issued_at = extension.created_at.unwrap_or(env.block.time);

    let data = NodData {
        nod_id: entity.nod_id.clone(),
        worldwide_day: entity.worldwide_day,
        settlement_currency: entity.settlement_currency.clone(),
        symbolic_rate: entity.symbolic_rate,
        floor_rate: entity.floor_rate,
        nominal_price: entity.nominal_price,
        issuance_price: entity.issuance_price,
        gratis_load_minor: entity.gratis_load_minor,
        floor_price: entity.floor_price,
        state: entity.state.clone(),
        owner: entity.owner.clone(),
        issued_at: node_issued_at,
        qualified_at: entity.qualified_at,
        is_touch: entity.is_touch,
    };
    let token = NodNft {
        owner: owner_addr,
        token_uri: None, // todo populate
        extension: data,
    };

    let config = Cw721Config::<NodData, NodConfig>::default();
    config
        .nft_info
        .update(deps.storage, &token_id, |old| match old {
            Some(_) => Err(Cw721ContractError::Claimed {}),
            None => Ok(token),
        })?;
    config.increment_tokens(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "nod::submit")
        .add_attribute("token_id", token_id)
        .add_attribute("owner", owner)
        .add_attribute(
            "settlement_currency",
            entity.settlement_currency.clone().to_string(),
        )
        .add_attribute("symbolic_rate", entity.symbolic_rate.clone().to_string())
        .add_attribute("floor_rate", entity.floor_rate.clone().to_string())
        .add_attribute("nominal_price", entity.nominal_price.clone().to_string())
        .add_attribute("issuance_price", entity.issuance_price.clone().to_string())
        .add_attribute(
            "gratis_load_minor",
            entity.gratis_load_minor.clone().to_string(),
        )
        .add_attribute("floor_price", entity.floor_price.clone().to_string())
        .add_attribute("state", entity.state.clone().to_string())
        .add_attribute("issued_at", node_issued_at.clone().to_string())
        .add_attribute("is_touch", entity.is_touch.clone().to_string())
        .add_attribute(
            "qualified_at",
            entity
                .qualified_at
                .map(|t| t.seconds())
                .unwrap_or_default()
                .to_string(),
        ))
}

fn execute_burn(
    deps: DepsMut,
    _env: &Env,
    info: &MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    let config = Cw721Config::<NodData, NodConfig>::default();
    config.nft_info.remove(deps.storage, &token_id)?;
    config.decrement_tokens(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "nod::burn")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("token_id", token_id))
}

fn execute_update_tokens_to_qualified(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    price_threshold: Decimal,
) -> Result<Response, ContractError> {
    // Verify caller is the authorized price updater
    if PRICE_UPDATER
        .assert_owner(deps.storage, &info.sender)
        .is_err()
    {
        return Err(ContractError::Unauthorized {});
    }

    let config = Cw721Config::<NodData, NodConfig>::default();
    let mut updated_count = 0u32;
    let mut updated_tokens = Vec::<String>::new();

    let all_tokens: Vec<(String, NftInfo<NodData>)> = config
        .nft_info
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .filter_map(|item| match item {
            Ok((token_id, data)) => {
                if data.extension.state == State::Issued
                    && data.extension.floor_price <= price_threshold
                {
                    let mut new_data = data.clone();
                    new_data.extension.state = State::Qualified;
                    new_data.extension.qualified_at = Some(env.block.time);
                    Some((token_id, new_data))
                } else {
                    None
                }
            }
            Err(_) => None,
        })
        .collect();

    for (token_id, data) in all_tokens {
        config.nft_info.save(deps.storage, &token_id, &data)?;

        updated_count += 1;
        updated_tokens.push(token_id);
    }

    Ok(Response::new()
        .add_attribute("action", "nod::update_tokens_to_qualified")
        .add_attribute("price_threshold", price_threshold.to_string())
        .add_attribute("updated_count", updated_count.to_string())
        .add_event(
            Event::new("nod::tokens_qualified")
                .add_attribute("updated_count", updated_count.to_string())
                .add_attribute("updated_tokens", updated_tokens.join(",")),
        ))
}

fn execute_update_price_updater(
    deps: DepsMut,
    _env: &Env,
    info: &MessageInfo,
    price_updater: Option<String>,
) -> Result<Response, ContractError> {
    // Check authorization - only creator can update price updater
    outbe_nft::execute::assert_creator(deps.storage, &info.sender)
        .map_err(|_| ContractError::Unauthorized {})?;

    // Update or remove the price updater
    match price_updater.clone() {
        Some(updater) => {
            let updater_addr = deps.api.addr_validate(&updater)?;
            PRICE_UPDATER.initialize_owner(deps.storage, deps.api, Some(updater_addr.as_str()))?;
        }
        None => {
            // Remove the price updater by initializing with None
            PRICE_UPDATER.initialize_owner(deps.storage, deps.api, None)?;
        }
    }

    Ok(Response::new()
        .add_attribute("action", "nod::update_price_updater")
        .add_event(
            Event::new("nod::price_updater_updated")
                .add_attribute("price_updater", price_updater.unwrap_or("none".to_string()))
                .add_attribute("updated_by", info.sender.clone()),
        ))
}

#[cfg(feature = "demo")]
fn execute_burn_all(
    deps: DepsMut,
    _env: &Env,
    info: &MessageInfo,
    batch_size: Option<usize>,
) -> Result<Response, ContractError> {
    let config = Cw721Config::<NodData, NodConfig>::default();
    // TODO verify ownership
    // let token = config.nft_info.load(deps.storage, &token_id)?;
    // check_can_send(deps.as_ref(), env, info.sender.as_str(), &token)?;

    config.clean_tokens(deps.storage, batch_size)?;

    Ok(Response::new()
        .add_attribute("action", "nod::burn_all")
        .add_event(Event::new("nod::burn_all").add_attribute("sender", info.sender.to_string())))
}
