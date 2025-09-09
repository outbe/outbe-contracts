use crate::agent_common::*;
use agent_nra::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use crate::state::CONFIG;
use crate::types::Config;
use cosmwasm_std::{entry_point, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "outbe.net:agent-nra";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let cfg = Config {
        owner: info.sender.clone(),
        agent_registry: msg.agent_registry,
        paused: msg.paused.unwrap_or(false),
        last_token_id: 1u32,
    };

    CONFIG.save(deps.storage, &cfg)?;

    Ok(Response::new()
        .add_attribute("action", "agent-nra::instantiate")
        .add_attribute("version", CONTRACT_VERSION))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    match msg {
        MigrateMsg::Migrate {} => Ok(Response::new()),
    }
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // Agent
        ExecuteMsg::SubmitAgent { id } => exec_submit_agent(deps, env, info, id),
        ExecuteMsg::EditAgent { agent } => exec_edit_agent(deps, env, info, agent),
        ExecuteMsg::HoldAgent { address } => exec_hold_agent(deps, env, info, address),
        ExecuteMsg::BanAgent { address } => exec_ban_agent(deps, env, info, address),
        ExecuteMsg::ActivateAgent { address } => exec_activate_agent(deps, env, info, address),
        ExecuteMsg::ResignAgent {} => exec_resign_agent(deps, env, info),
    }
}
