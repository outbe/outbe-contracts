use crate::msg::{ExecuteMsg, MigrateMsg};
use agent_common::msg::InstantiateMsg;
use agent_common::state::{Config, CONFIG, AGENTS};
use agent_common::types::{AgentExt, AgentStatus, ExternalWallet};
use agent_nra::agent_common::*;
use agent_nra::error::ContractError;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;
const CONTRACT_NAME: &str = "outbe.net:agent-iba";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    match msg {
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
        ExecuteMsg::EditAdditionalWallets { additional_wallets } => {
            exec_edit_additional_wallets(deps, env, info, additional_wallets)
        }
    }
}

pub fn exec_edit_additional_wallets(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    additional_wallets: Option<Vec<ExternalWallet>>,
) -> Result<Response, ContractError> {
    // Load the agent for the sender
    let mut agent = AGENTS
        .may_load(deps.storage, info.sender.clone())?
        .ok_or(ContractError::AgentNotFound {})?;

    // Check if the agent status is Active
    if agent.status != AgentStatus::Active {
        return Err(ContractError::Unauthorized);
    }

    // Check if this is an IBA agent and update only additional_wallets
    if let AgentExt::Iba { additional_wallets: current_wallets, .. } = &mut agent.ext {
        *current_wallets = additional_wallets.clone();
    } else {
        return Err(ContractError::Unauthorized);
    }

    // Update timestamp
    agent.updated_at = env.block.time;

    // Save the updated agent (keep status as Active, no review needed)
    AGENTS.save(deps.storage, info.sender.clone(), &agent)?;

    Ok(Response::new()
        .add_attribute("action", "edit_iba_additional_wallets")
        .add_attribute("agent_wallet", info.sender.to_string())
        .add_attribute("updated_at", agent.updated_at.to_string())
        .add_attribute(
            "additional_wallets_count",
            additional_wallets.as_ref().map_or(0, |w| w.len()).to_string(),
        ))
}
