use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, SubmitExtension};
use crate::types::{NodConfig, NodData, NodNft};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{DepsMut, Env, Event, MessageInfo, Response};
use outbe_nft::error::Cw721ContractError;
use outbe_nft::state::{CollectionInfo, Cw721Config};

const CONTRACT_NAME: &str = "outbe.net:nod";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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
        ExecuteMsg::BurnAll {} => execute_burn_all(deps, &env, &info),
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

    let data = NodData {
        nod_id: entity.nod_id.clone(),
        settlement_currency: entity.settlement_currency.clone(),
        symbolic_rate: entity.symbolic_rate,
        floor_rate: entity.floor_rate,
        nominal_price_minor: entity.nominal_price_minor,
        issuance_price_minor: entity.issuance_price_minor,
        gratis_load_minor: entity.gratis_load_minor,
        floor_price_minor: entity.floor_price_minor,
        state: entity.state.clone(),
        owner: entity.owner.clone(),
        issued_at: extension.created_at.unwrap_or(env.block.time),
        qualified_at: entity.qualified_at,
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
        .add_attribute("owner", owner))
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

fn execute_burn_all(
    deps: DepsMut,
    _env: &Env,
    info: &MessageInfo,
) -> Result<Response, ContractError> {
    let config = Cw721Config::<NodData, NodConfig>::default();
    // TODO verify ownership
    // let token = config.nft_info.load(deps.storage, &token_id)?;
    // check_can_send(deps.as_ref(), env, info.sender.as_str(), &token)?;

    config.nft_info.clear(deps.storage);
    config.token_count.save(deps.storage, &0u64)?;

    Ok(Response::new()
        .add_attribute("action", "nod::burn_all")
        .add_event(Event::new("nod::burn_all").add_attribute("sender", info.sender.to_string())))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new())
}
