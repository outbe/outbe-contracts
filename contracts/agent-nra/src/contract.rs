use crate::agent_common::*;
use crate::error::ContractError;
use crate::msg::{AgentMsg, ApplicationMsg, ExecuteMsg, InstantiateMsg, MigrateMsg};
use crate::state::{Config, ThresholdConfig, APPLICATIONS, APPLICATION_VOTES, CONFIG};
use crate::types::{Application, ApplicationInput, ApplicationStatus, Vote};
use agent_common::state::AGENTS;
use agent_common::types::{AgentExt, AgentStatus, AgentType};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Addr, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
const CONTRACT_NAME: &str = "outbe.net:agent-nra";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let default_thresholds = ThresholdConfig {
        nra: 3,
        cra: 1,
        rfa: 1,
        iba: 1,
        cca: 1,
    };

    let bootstrap_voters: Vec<Addr> = msg
        .bootstrap_voters
        .unwrap_or_default()
        .into_iter()
        .map(Addr::unchecked)
        .collect();

    let cfg = Config {
        owner: info.sender.clone(),
        thresholds: msg.thresholds.unwrap_or(default_thresholds),
        paused: msg.paused.unwrap_or(false),
        last_application_id: 0u32,
        bootstrap_voters,
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let registry = env.contract.address.clone();
    match msg {
        ExecuteMsg::Application(app_msg) => match app_msg {
            ApplicationMsg::CreateApplication { application } => {
                exec_add_application(deps, env, info, *application)
            }
            ApplicationMsg::EditApplication { id, application } => {
                exec_update_application(deps, env, info, id, *application)
            }
            ApplicationMsg::VoteApplication {
                id,
                approve,
                reason,
            } => exec_vote_application(deps, env, info, id, approve, reason),
            ApplicationMsg::HoldApplication { id } => exec_hold_application(deps, env, info, id),

            ApplicationMsg::AddBootstrapVoter { address } => {
                exec_add_bootstrap_voter(deps, info, address)
            }
            ApplicationMsg::RemoveBootstrapVoter { address } => {
                exec_remove_bootstrap_voter(deps, info, address)
            }
        },

        ExecuteMsg::Agent(agent_msg) => match agent_msg {
            AgentMsg::SubmitAgent { id } => {
                exec_submit_agent(deps, env.clone(), info, id, registry)
            }
            AgentMsg::EditAgent { agent } => exec_edit_agent(deps, env, info, *agent),
            AgentMsg::HoldAgent { address } => exec_hold_agent(deps, env, info, address, registry),
            AgentMsg::BanAgent { address } => exec_ban_agent(deps, env, info, address, registry),
            AgentMsg::ActivateAgent { address } => {
                exec_activate_agent(deps, env, info, address, registry)
            }
            AgentMsg::ResignAgent {} => exec_resign_agent(deps, env, info),
        },
    }
}

pub fn exec_add_application(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    input: ApplicationInput,
) -> Result<Response, ContractError> {
    let wallet = info.sender;

    let mut config = CONFIG.load(deps.storage)?;

    let now = env.block.time;
    let id = config.last_application_id;

    let application = Application {
        id,
        application_type: input.application_type,
        wallet: wallet.clone(),
        name: input.name,
        email: input.email,
        jurisdictions: input.jurisdictions,
        endpoint: input.endpoint,
        metadata_json: input.metadata_json,
        docs_uri: input.docs_uri,
        discord: input.discord,
        status: ApplicationStatus::InReview,
        avg_cu: input.avg_cu,
        submitted_at: now,
        updated_at: now,
        ext: input.ext,
    };
    APPLICATIONS.save(deps.storage, id.to_string(), &application)?;

    config.last_application_id = id.saturating_add(1);
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "application::add")
        .add_attribute("application_id", id.to_string())
        .add_attribute("wallet", wallet.to_string()))
}

pub fn exec_update_application(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: String,
    input: ApplicationInput,
) -> Result<Response, ContractError> {
    let mut existing_aplication = APPLICATIONS
        .may_load(deps.storage, id.clone())?
        .ok_or(ContractError::ApplicationNotFound {})?;

    if info.sender != existing_aplication.wallet {
        return Err(ContractError::OwnerError {});
    }

    if !matches!(
        existing_aplication.status,
        ApplicationStatus::InReview | ApplicationStatus::OnHold
    ) {
        return Err(ContractError::InvalidApplicationStatus {});
    }

    existing_aplication.name = input.name.trim().to_string();
    existing_aplication.email = input.email.trim().to_string();
    existing_aplication.jurisdictions = input.jurisdictions;
    existing_aplication.endpoint = input.endpoint;
    existing_aplication.metadata_json = input.metadata_json;
    existing_aplication.docs_uri = input.docs_uri;
    existing_aplication.discord = input.discord;
    existing_aplication.status = ApplicationStatus::InReview;
    existing_aplication.avg_cu = input.avg_cu;
    existing_aplication.updated_at = env.block.time;

    APPLICATIONS.save(deps.storage, id.clone(), &existing_aplication)?;

    Ok(Response::new()
        .add_attribute("action", "application::update")
        .add_attribute("application_id", id)
        .add_attribute("wallet", existing_aplication.wallet.to_string()))
}

pub fn exec_vote_application(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: String,
    approve: bool,
    reason: Option<String>,
) -> Result<Response, ContractError> {
    // Check if sender is an active NRA agent
    ensure_active_nra_agent(&deps, &info.sender)?;

    let mut application = APPLICATIONS
        .may_load(deps.storage, id.clone())?
        .ok_or(ContractError::ApplicationNotFound {})?;

    if info.sender == application.wallet {
        return Err(ContractError::SelfVote {});
    }

    //check prefered nra
    if application.application_type == AgentType::Cra {
        ensure_preferred_nra(&application.ext, &info.sender)?;
    }

    if APPLICATION_VOTES.has(deps.storage, (id.as_str(), &info.sender)) {
        return Err(ContractError::AlreadyVoted {});
    }

    if matches!(
        application.status,
        ApplicationStatus::Approved | ApplicationStatus::Rejected
    ) {
        return Err(ContractError::AlreadyFinalized {});
    }

    let vote = Vote {
        address: info.sender.to_string(),
        approve,
        reason,
        application_id: id.clone(),
        at: env.block.time,
    };

    APPLICATION_VOTES.save(deps.storage, (id.as_str(), &info.sender), &vote)?;

    let threshold = get_threshold(deps.as_ref(), &application.application_type)?;
    let (approvals, rejects) = count_votes(deps.as_ref(), id.as_str())?;

    // Update Agent Status
    let new_status = if approvals >= threshold {
        Some(ApplicationStatus::Approved)
    } else if rejects >= 1 {
        Some(ApplicationStatus::Rejected)
    } else {
        None
    };

    let response = Response::new()
        .add_attribute("action", "application::vote_application")
        .add_attribute("application_id", id.clone())
        .add_attribute("approved", approve.to_string());

    if let Some(status) = new_status {
        application.status = status.clone();
        application.updated_at = env.block.time;
        APPLICATIONS.save(deps.storage, id.clone(), &application)?;
    }

    Ok(response)
}

pub fn exec_hold_application(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: String,
) -> Result<Response, ContractError> {
    // Check if sender is an active NRA agent
    ensure_active_nra_agent(&deps, &info.sender)?;

    let mut existing_aplication = APPLICATIONS
        .may_load(deps.storage, id.clone())?
        .ok_or(ContractError::ApplicationNotFound {})?;

    existing_aplication.status = ApplicationStatus::OnHold;
    existing_aplication.updated_at = env.block.time;

    APPLICATIONS.save(deps.storage, id.clone(), &existing_aplication)?;

    Ok(Response::new()
        .add_attribute("action", "application::hold")
        .add_attribute("application_id", id)
        .add_attribute("wallet", existing_aplication.wallet.to_string()))
}

pub fn exec_add_bootstrap_voter(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    let mut cfg = CONFIG.load(deps.storage)?;
    ensure_owner(&cfg, &info.sender)?;

    let voter = deps.api.addr_validate(&address)?;

    if cfg.bootstrap_voters.contains(&voter) {
        return Err(ContractError::InvalidBootstrapAction {});
    }

    cfg.bootstrap_voters.push(voter.clone());
    CONFIG.save(deps.storage, &cfg)?;

    Ok(Response::new()
        .add_attribute("action", "agent-nra::add_bootstrap_voter")
        .add_attribute("voter", voter))
}

pub fn exec_remove_bootstrap_voter(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    let mut cfg = CONFIG.load(deps.storage)?;
    ensure_owner(&cfg, &info.sender)?;

    let voter = deps.api.addr_validate(&address)?;

    if let Some(pos) = cfg.bootstrap_voters.iter().position(|a| a == voter) {
        cfg.bootstrap_voters.remove(pos);
    } else {
        return Err(ContractError::InvalidBootstrapAction {});
    }

    CONFIG.save(deps.storage, &cfg)?;

    Ok(Response::new()
        .add_attribute("action", "agent-nra::remove_bootstrap_voter")
        .add_attribute("voter", voter))
}

pub fn count_votes(deps: Deps, id: &str) -> StdResult<(u8, u8)> {
    use cosmwasm_std::Order;

    let mut approvals = 0;
    let mut rejects = 0;

    for item in APPLICATION_VOTES
        .prefix(id)
        .range(deps.storage, None, None, Order::Ascending)
    {
        let (_, v) = item?;
        if v.approve {
            approvals += 1
        } else {
            rejects += 1
        }
    }
    Ok((approvals, rejects))
}

pub fn get_threshold(deps: Deps, agent_type: &AgentType) -> StdResult<u8> {
    let thresholds = CONFIG.load(deps.storage)?.thresholds;
    Ok(match agent_type {
        AgentType::Nra => thresholds.nra,
        AgentType::Cra => thresholds.cra,
        AgentType::Rfa => thresholds.rfa,
        AgentType::Iba => thresholds.iba,
    })
}

pub fn ensure_active_nra_agent(deps: &DepsMut, sender: &Addr) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Check if sender is in bootstrap voters
    if config.bootstrap_voters.contains(sender) {
        return Ok(());
    }

    let agent = AGENTS
        .may_load(deps.storage, sender.clone())?
        .ok_or(ContractError::OnlyActiveNra {})?;

    if !matches!(agent.status, AgentStatus::Active) {
        return Err(ContractError::OnlyActiveNra {});
    }

    Ok(())
}
fn ensure_owner(cfg: &Config, sender: &cosmwasm_std::Addr) -> Result<(), ContractError> {
    if cfg.owner != sender {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

fn ensure_preferred_nra(ext: &Option<AgentExt>, sender: &Addr) -> Result<(), ContractError> {
    if let Some(AgentExt::Cra {
        preferred_nra: Some(list),
        ..
    }) = ext
    {
        if !list.is_empty() {
            let allowed = list.iter().any(|w| w == sender);

            if !allowed {
                return Err(ContractError::OnlyPreferredNra {});
            }
        }
    }
    Ok(())
}
