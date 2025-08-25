use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{AGENTS, AGENT_VOTES, CONFIG};
use crate::types::{Agent, AgentInput, AgentStatus, Config, Vote};
use cosmwasm_std::{entry_point, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "outbe.net:agent";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = info.sender.clone();

    let cfg = Config {
        owner,
        threshold: msg.threshold.unwrap_or(3),
        paused: msg.paused.unwrap_or(false),
        last_token_id: Uint128::one(),
    };
    CONFIG.save(deps.storage, &cfg)?;

    Ok(Response::new()
        .add_attribute("action", "agent::instantiate")
        .add_attribute("version", CONTRACT_VERSION)
        .add_attribute("threshold", cfg.threshold.to_string()))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateAgent { agent } => execute_add_agent(deps, env, info, agent),
        ExecuteMsg::UpdateAgent { id, agent } => execute_update_agent(deps, env, info, id, agent),
        ExecuteMsg::VoteAgent {
            id,
            approve,
            reason,
        } => exec_vote_agent(deps, env, info, id, approve, reason),
    }
}

pub fn execute_add_agent(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    input: AgentInput,
) -> Result<Response, ContractError> {
    let wallet = info.sender;

    let mut config = CONFIG.load(deps.storage)?;

    let now = env.block.time;
    let id = config.last_token_id.to_string();

    let agent = Agent {
        id: id.clone(),
        agent_type: input.agent_type,
        wallet: wallet.clone().to_string(),
        name: input.name,
        email: input.email,
        jurisdictions: input.jurisdictions,
        endpoint: input.endpoint,
        metadata_json: input.metadata_json,
        docs_uri: input.docs_uri,
        discord: input.discord,
        status: AgentStatus::Pending,
        avg_cu: input.avg_cu,
        submitted_at: now,
        updated_at: now,
    };
    AGENTS.save(deps.storage, id.clone(), &agent)?;

    config.last_token_id += Uint128::one();
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "agent::add")
        .add_attribute("agent_id", id)
        .add_attribute("wallet", wallet.to_string()))
}

pub fn execute_update_agent(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: String,
    input: AgentInput,
) -> Result<Response, ContractError> {
    let mut founded_agent = AGENTS
        .may_load(deps.storage, id.clone())?
        .ok_or(ContractError::AgentNotFound {})?;

    if info.sender.to_string() != founded_agent.wallet {
        return Err(ContractError::OwnerError {});
    }

    founded_agent.agent_type = input.agent_type;
    founded_agent.name = input.name.trim().to_string();
    founded_agent.email = input.email.trim().to_string();
    founded_agent.jurisdictions = input.jurisdictions;
    founded_agent.endpoint = input.endpoint;
    founded_agent.metadata_json = input.metadata_json;
    founded_agent.docs_uri = input.docs_uri;
    founded_agent.discord = input.discord;
    founded_agent.status = input.status;
    founded_agent.avg_cu = input.avg_cu;
    founded_agent.updated_at = env.block.time;

    AGENTS.save(deps.storage, id.clone(), &founded_agent)?;

    Ok(Response::new()
        .add_attribute("action", "agent::update")
        .add_attribute("agent_id", id)
        .add_attribute("wallet", founded_agent.wallet.to_string()))
}

pub fn exec_vote_agent(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: String,
    approve: bool,
    reason: Option<String>,
) -> Result<Response, ContractError> {
    let mut agent = AGENTS
        .may_load(deps.storage, id.clone())?
        .ok_or(ContractError::AgentNotFound {})?;

    if info.sender.to_string() == agent.wallet {
        return Err(ContractError::SelfVote {});
    }

    if AGENT_VOTES.has(deps.storage, (id.as_str(), &info.sender)) {
        return Err(ContractError::AlreadyVoted {});
    }

    if matches!(agent.status, AgentStatus::Approved | AgentStatus::Rejected) {
        return Err(ContractError::AlreadyFinalized {});
    }

    let vote = Vote {
        address: info.sender.to_string(),
        approve,
        reason,
        at: env.block.time,
    };

    AGENT_VOTES.save(deps.storage, (id.as_str(), &info.sender), &vote)?;

    let threshold = CONFIG.load(deps.storage)?.threshold as usize;
    let (approvals, rejects) = count_votes(deps.as_ref(), id.as_str())?;

    // Update Agent Status
    let new_status = if approvals >= threshold {
        Some(AgentStatus::Approved)
    } else if rejects >= 1 {
        Some(AgentStatus::Rejected)
    } else {
        None
    };

    if new_status.is_some() {
        agent.status = new_status.unwrap();
        agent.updated_at = env.block.time;
        AGENTS.save(deps.storage, id.clone(), &agent)?;
    }

    Ok(Response::new()
        .add_attribute("action", "agent::vote_agent")
        .add_attribute("agent_id", id)
        .add_attribute("status", format!("{:?}", agent.status))
        .add_attribute("approved", approve.to_string()))
}

pub fn count_votes(deps: Deps, id: &str) -> StdResult<(usize, usize)> {
    use cosmwasm_std::Order;

    let mut approvals = 0usize;
    let mut rejects = 0usize;

    for item in AGENT_VOTES
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
