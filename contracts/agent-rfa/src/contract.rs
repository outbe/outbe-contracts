use crate::msg::{ExecuteMsg, MigrateMsg};
use agent_common::msg::InstantiateMsg;
use agent_common::state::{Config, CONFIG};
use agent_nra::agent_common::*;
use agent_nra::error::ContractError;
use cosmwasm_std::{entry_point, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "outbe.net:agent-rfa";
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
        agent_registry: msg.application_registry_addr,
        paused: msg.paused.unwrap_or(false),
        last_token_id: 1u32,
    };

    CONFIG.save(deps.storage, &cfg)?;

    Ok(Response::new()
        .add_attribute("action", "agent-rfa::instantiate")
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
    let cfg = CONFIG.load(deps.storage)?;

    match msg {
        // Agent
        ExecuteMsg::SubmitAgent { id } => {
            exec_submit_agent(deps, env, info, id, cfg.agent_registry)
        }
        ExecuteMsg::EditAgent { agent } => exec_edit_agent(deps, env, info, *agent),
        ExecuteMsg::HoldAgent { address } => {
            exec_hold_agent(deps, env, info, address, cfg.agent_registry)
        }
        ExecuteMsg::BanAgent { address } => {
            exec_ban_agent(deps, env, info, address, cfg.agent_registry)
        }
        ExecuteMsg::ActivateAgent { address } => {
            exec_activate_agent(deps, env, info, address, cfg.agent_registry)
        }
        ExecuteMsg::ResignAgent {} => exec_resign_agent(deps, env, info),
    }
}
