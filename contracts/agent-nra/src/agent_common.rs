use crate::contract::ensure_active_nra_agent;
use crate::error::ContractError;
use crate::state::{APPLICATIONS};
use crate::types::{ApplicationStatus};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use agent_common::state::AGENTS;
use agent_common::types::{Agent, AgentInput, AgentStatus};

pub fn exec_submit_agent(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: String,
) -> Result<Response, ContractError> {
    let existing_application = APPLICATIONS
        .may_load(deps.storage, id.clone())?
        .ok_or(ContractError::ApplicationNotFound {})?;
    // Check owner
    if info.sender != existing_application.wallet {
        return Err(ContractError::ApplicationOwnerError {});
    }
    //Check status
    if !matches!(existing_application.status, ApplicationStatus::Approved) {
        return Err(ContractError::ApplicationNotApproved {});
    }

    let agent = Agent {
        wallet: existing_application.wallet.clone(),
        name: existing_application.name,
        email: existing_application.email,
        agent_type: existing_application.application_type,
        jurisdictions: existing_application.jurisdictions,
        endpoint: existing_application.endpoint.clone(),
        metadata_json: existing_application.metadata_json.clone(),
        docs_uri: existing_application.docs_uri.clone(),
        discord: existing_application.discord.clone(),
        status: AgentStatus::Active,
        avg_cu: existing_application.avg_cu,
        submitted_at: env.block.time,
        updated_at: env.block.time,
        ext: existing_application.ext.unwrap().clone(),
    };

    AGENTS.save(deps.storage, existing_application.wallet.clone(), &agent)?;

    Ok(Response::new()
        .add_attribute("action", "agent::submit")
        .add_attribute("application_id", id)
        .add_attribute("wallet", existing_application.wallet.to_string()))
}

pub fn exec_edit_agent(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    agent_input: AgentInput,
) -> Result<Response, ContractError> {
    let mut agent = AGENTS
        .may_load(deps.storage, info.sender.clone())?
        .ok_or(ContractError::AgentNotFound {})?;

    // Check if the agent status is Active
    if agent.status != AgentStatus::Active {
        return Err(ContractError::Unauthorized);
    }
    // Update agent fields from the input
    agent.name = agent_input.name.trim().to_string();
    agent.email = agent_input.email.trim().to_string();
    agent.jurisdictions = agent_input.jurisdictions;
    agent.endpoint = agent_input.endpoint;
    agent.metadata_json = agent_input.metadata_json;
    agent.docs_uri = agent_input.docs_uri;
    agent.discord = agent_input.discord;
    agent.avg_cu = agent_input.avg_cu;
    agent.ext = agent_input.ext;
    agent.updated_at = env.block.time;
    agent.status = AgentStatus::InReview;

    // Save the updated agent
    AGENTS.save(deps.storage, info.sender.clone(), &agent)?;

    Ok(Response::new()
        .add_attribute("action", "edit_agent")
        .add_attribute("agent_wallet", info.sender.to_string())
        .add_attribute("updated_at", agent.updated_at.to_string()))
}
pub fn exec_hold_agent(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    ensure_active_nra_agent(&deps, &info.sender)?;

    // Parse address string
    let agent_addr = deps.api.addr_validate(&address)?;

    // Load agent
    let mut agent = AGENTS
        .may_load(deps.storage, agent_addr.clone())?
        .ok_or(ContractError::AgentNotFound {})?;

    // Check if agent status is Active before hold
    if !matches!(agent.status, AgentStatus::Active) {
        return Err(ContractError::InvalidAgentStatus {});
    }

    let old_status = agent.status.clone();
    agent.status = AgentStatus::OnHold;
    agent.updated_at = env.block.time;

    // Save updated agent
    AGENTS.save(deps.storage, agent_addr.clone(), &agent)?;

    Ok(Response::new()
        .add_attribute("action", "hold_agent")
        .add_attribute("address", address)
        .add_attribute("old_status", format!("{:?}", old_status))
        .add_attribute("new_status", "OnHold")
        .add_attribute("updated_at", agent.updated_at.to_string()))
}

pub fn exec_ban_agent(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    ensure_active_nra_agent(&deps, &info.sender)?;
    // Parse address string
    let agent_addr = deps.api.addr_validate(&address)?;

    // Load agent
    let mut agent = AGENTS
        .may_load(deps.storage, agent_addr.clone())?
        .ok_or(ContractError::AgentNotFound {})?;

    // Check if agent status is OnHold before banning
    if !matches!(agent.status, AgentStatus::OnHold) {
        return Err(ContractError::InvalidAgentStatus {});
    }

    let old_status = agent.status.clone();
    agent.status = AgentStatus::Blacklisted;
    agent.updated_at = env.block.time;

    // Save updated agent
    AGENTS.save(deps.storage, agent_addr.clone(), &agent)?;

    Ok(Response::new()
        .add_attribute("action", "ban_agent")
        .add_attribute("address", address)
        .add_attribute("old_status", format!("{:?}", old_status))
        .add_attribute("new_status", "Blacklisted")
        .add_attribute("updated_at", agent.updated_at.to_string()))
}

pub fn exec_activate_agent(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    ensure_active_nra_agent(&deps, &info.sender)?;

    // Parse address string
    let agent_addr = deps.api.addr_validate(&address)?;

    // Load agent
    let mut agent = AGENTS
        .may_load(deps.storage, agent_addr.clone())?
        .ok_or(ContractError::AgentNotFound {})?;

    // Check if agent status is OnHold or InReview before activating
    if !matches!(agent.status, AgentStatus::OnHold | AgentStatus::InReview) {
        return Err(ContractError::InvalidAgentStatus {});
    }

    let old_status = agent.status.clone();

    agent.status = AgentStatus::Active;
    agent.updated_at = env.block.time;

    // Save updated agent
    AGENTS.save(deps.storage, agent_addr.clone(), &agent)?;

    Ok(Response::new()
        .add_attribute("action", "activate_agent")
        .add_attribute("address", address)
        .add_attribute("old_status", format!("{:?}", old_status))
        .add_attribute("new_status", "Active")
        .add_attribute("updated_at", agent.updated_at.to_string()))
}

pub fn exec_resign_agent(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // Load agent for the sender
    let mut agent = AGENTS
        .may_load(deps.storage, info.sender.clone())?
        .ok_or(ContractError::AgentNotFound {})?;

    // Only Active agents can resign
    if agent.status != AgentStatus::Active {
        return Err(ContractError::Unauthorized);
    }

    let old_status = agent.status.clone();
    agent.status = AgentStatus::Resigned;
    agent.updated_at = env.block.time;

    // Save updated agent
    AGENTS.save(deps.storage, info.sender.clone(), &agent)?;

    Ok(Response::new()
        .add_attribute("action", "resign_agent")
        .add_attribute("agent_wallet", info.sender.to_string())
        .add_attribute("old_status", format!("{:?}", old_status))
        .add_attribute("new_status", "Resigned")
        .add_attribute("updated_at", agent.updated_at.to_string()))
}
