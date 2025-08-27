use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{ACCOUNTS, AGENTS, AGENT_VOTES, CONFIG};
use crate::types::{Account, AccountInput, Agent, AgentInput, AgentStatus, Config, Vote};
use cosmwasm_std::{entry_point, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "outbe.net:agent-registry";
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
        last_token_id: 1u32,
    };
    CONFIG.save(deps.storage, &cfg)?;

    Ok(Response::new()
        .add_attribute("action", "agent-registry::instantiate")
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
        ExecuteMsg::UpdateAccount { account } => execute_update_account(deps, env, info, account),
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
    let id = config.last_token_id;

    let agent = Agent {
        id,
        agent_type: input.agent_type,
        wallet: wallet.clone(),
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
    AGENTS.save(deps.storage, id.to_string(), &agent)?;

    config.last_token_id = id.saturating_add(1);
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "agent-registry::add")
        .add_attribute("agent_id", id.to_string())
        .add_attribute("wallet", wallet.to_string()))
}

pub fn execute_update_agent(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: String,
    input: AgentInput,
) -> Result<Response, ContractError> {
    let mut existing_agent = AGENTS
        .may_load(deps.storage, id.clone())?
        .ok_or(ContractError::AgentNotFound {})?;

    if info.sender != existing_agent.wallet {
        return Err(ContractError::OwnerError {});
    }

    existing_agent.agent_type = input.agent_type;
    existing_agent.name = input.name.trim().to_string();
    existing_agent.email = input.email.trim().to_string();
    existing_agent.jurisdictions = input.jurisdictions;
    existing_agent.endpoint = input.endpoint;
    existing_agent.metadata_json = input.metadata_json;
    existing_agent.docs_uri = input.docs_uri;
    existing_agent.discord = input.discord;
    existing_agent.status = input.status;
    existing_agent.avg_cu = input.avg_cu;
    existing_agent.updated_at = env.block.time;

    AGENTS.save(deps.storage, id.clone(), &existing_agent)?;

    Ok(Response::new()
        .add_attribute("action", "agent-registry::update")
        .add_attribute("agent_id", id)
        .add_attribute("wallet", existing_agent.wallet.to_string()))
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

    if info.sender == agent.wallet {
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

    let mut response = Response::new()
        .add_attribute("action", "agent-registry::vote_agent")
        .add_attribute("agent_id", id.clone())
        .add_attribute("approved", approve.to_string());

    if let Some(status) = new_status {
        agent.status = status.clone();
        agent.updated_at = env.block.time;
        AGENTS.save(deps.storage, id.clone(), &agent)?;

        // Generate account if approved
        if matches!(status, AgentStatus::Approved) {
            create_account_from_agent(deps, env, &agent)?;
            response = response
                .add_attribute("account_created", "true")
                .add_attribute("account_address", agent.wallet.to_string());
        }
    }

    Ok(response)
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

fn create_account_from_agent(deps: DepsMut, env: Env, agent: &Agent) -> Result<(), ContractError> {
    // Create account from approved agent, using agent's wallet as key
    let account = Account {
        agent_type: agent.agent_type.clone(),
        name: agent.name.clone(),
        email: agent.email.clone(),
        jurisdictions: agent.jurisdictions.clone(),
        endpoint: agent.endpoint.clone(),
        metadata_json: agent.metadata_json.clone(),
        docs_uri: agent.docs_uri.clone(),
        discord: agent.discord.clone(),
        status: AgentStatus::Approved,
        avg_cu: agent.avg_cu,
        submitted_at: env.block.time,
        updated_at: env.block.time,
    };

    ACCOUNTS.save(deps.storage, agent.wallet.clone(), &account)?;

    Ok(())
}

pub fn execute_update_account(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    account_input: AccountInput,
) -> Result<Response, ContractError> {
    // Check if account exists for this address
    let mut existing_account = ACCOUNTS
        .may_load(deps.storage, info.sender.clone())?
        .ok_or(ContractError::AccountNotFound {})?;

    // Update account fields
    existing_account.name = account_input.name.trim().to_string();
    existing_account.email = account_input.email.trim().to_string();
    existing_account.jurisdictions = account_input.jurisdictions;
    existing_account.endpoint = account_input.endpoint;
    existing_account.metadata_json = account_input.metadata_json;
    existing_account.docs_uri = account_input.docs_uri;
    existing_account.discord = account_input.discord;
    existing_account.avg_cu = account_input.avg_cu;
    existing_account.updated_at = env.block.time;

    // Save updated account
    ACCOUNTS.save(deps.storage, info.sender.clone(), &existing_account)?;

    Ok(Response::new()
        .add_attribute("action", "agent-registry::update_account")
        .add_attribute("account_address", info.sender.to_string())
        .add_attribute("updated_at", existing_account.updated_at.to_string()))
}
