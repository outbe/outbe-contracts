use crate::msg::{
    ApplicationResponse, ApplicationVotesResponse, ListAllApplicationResponse, NraAccessResponse,
};
use crate::state::{APPLICATIONS, APPLICATION_VOTES, CONFIG};
use crate::types::{Application, Vote};
use agent_common::msg::{AgentResponse, ListAllAgentsResponse};
use agent_common::query::{query_agent_by_address, query_all_agents};
use agent_common::state::AGENTS;
use agent_common::types::AgentStatus;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{to_json_binary, Addr, Binary, Deps, Env, Order, StdResult};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cw_storage_plus::Bound;

pub const DEFAULT_LIMIT: u32 = 10;
pub const MAX_LIMIT: u32 = 1000;
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // Application
    #[returns(ListAllApplicationResponse)]
    ListAllApplications {
        start_after: Option<String>,
        limit: Option<u32>,
        query_order: Option<Order>,
    },
    #[returns(ApplicationResponse)]
    GetApplicationById { id: String },
    #[returns(ListAllApplicationResponse)]
    QueryApplicationByAddress {
        address: String,
        start_after: Option<String>,
        limit: Option<u32>,
        query_order: Option<Order>,
    },
    #[returns(ApplicationVotesResponse)]
    QueryVotesByApplication { id: String },
    #[returns(ApplicationVotesResponse)]
    QueryVotesByAddress { address: Addr },

    // Agent
    #[returns(AgentResponse)]
    GetAgentByAddress { address: Addr },

    #[returns(ListAllAgentsResponse)]
    ListAllAgents {
        start_after: Option<Addr>,
        limit: Option<u32>,
        query_order: Option<Order>,
    },

    #[returns(NraAccessResponse)]
    EnsureActiveNra { address: Addr },
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ListAllApplications {
            start_after,
            limit,
            query_order,
        } => to_json_binary(&query_all_applications(
            deps,
            start_after,
            limit,
            query_order,
        )?),
        QueryMsg::GetApplicationById { id } => to_json_binary(&query_by_id(deps, id)?),
        QueryMsg::QueryApplicationByAddress {
            address,
            start_after,
            limit,
            query_order,
        } => to_json_binary(&query_by_address(
            deps,
            address,
            start_after,
            limit,
            query_order,
        )?),
        QueryMsg::QueryVotesByApplication { id } => {
            to_json_binary(&query_votes_by_application(deps, id)?)
        }
        QueryMsg::QueryVotesByAddress { address } => {
            to_json_binary(&query_votes_by_address(deps, address)?)
        }

        QueryMsg::GetAgentByAddress { address } => {
            to_json_binary(&query_agent_by_address(deps, address)?)
        }
        QueryMsg::ListAllAgents {
            start_after,
            limit,
            query_order,
        } => to_json_binary(&query_all_agents(deps, start_after, limit, query_order)?),

        QueryMsg::EnsureActiveNra { address } => {
            to_json_binary(&query_ensure_active_nra(deps, address)?)
        }
    }
}

fn query_all_applications(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
    query_order: Option<Order>,
) -> StdResult<ListAllApplicationResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let order = query_order.unwrap_or(Order::Ascending);
    let (start, end) = match order {
        Order::Ascending => (start_after.as_deref().map(Bound::exclusive), None),
        Order::Descending => (None, start_after.as_deref().map(Bound::exclusive)),
    };

    let applications = APPLICATIONS
        .range(deps.storage, start, end, order)
        .take(limit)
        .map(|item| item.map(|item| item.1))
        .collect::<StdResult<Vec<Application>>>()?;

    Ok(ListAllApplicationResponse { applications })
}

fn query_by_id(deps: Deps, id: String) -> StdResult<ApplicationResponse> {
    let application = APPLICATIONS.may_load(deps.storage, id)?;

    Ok(ApplicationResponse { application })
}

fn query_by_address(
    deps: Deps,
    address: String,
    start_after: Option<String>,
    limit: Option<u32>,
    query_order: Option<Order>,
) -> StdResult<ListAllApplicationResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let order = query_order.unwrap_or(Order::Ascending);
    let (start, end) = match order {
        Order::Ascending => (start_after.as_deref().map(Bound::exclusive), None),
        Order::Descending => (None, start_after.as_deref().map(Bound::exclusive)),
    };

    let addr = deps.api.addr_validate(&address)?;

    let applications = APPLICATIONS
        .range(deps.storage, start, end, order)
        .filter_map(|item| match item {
            Ok((_id, agent)) if agent.wallet == addr => Some(Ok(agent)),
            Ok(_) => None,
            Err(e) => Some(Err(e)),
        })
        .take(limit)
        .collect::<StdResult<Vec<Application>>>()?;

    Ok(ListAllApplicationResponse { applications })
}

pub fn query_votes_by_application(deps: Deps, id: String) -> StdResult<ApplicationVotesResponse> {
    let votes: Vec<Vote> = APPLICATION_VOTES
        .prefix(id.as_str())
        .range(deps.storage, None, None, Order::Ascending)
        .map(|res| {
            let (_voter_addr, v) = res?;
            Ok(v)
        })
        .collect::<StdResult<Vec<Vote>>>()?;

    Ok(ApplicationVotesResponse { votes })
}

pub fn query_votes_by_address(deps: Deps, address: Addr) -> StdResult<ApplicationVotesResponse> {
    let votes: Vec<Vote> = APPLICATION_VOTES
        .range(deps.storage, None, None, Order::Ascending)
        .filter_map(|res| match res {
            Ok(((_agent_id, voter_address), vote)) if voter_address == address => Some(Ok(vote)),
            Ok(_) => None, // Этот голос не от нашего адреса
            Err(e) => Some(Err(e)),
        })
        .collect::<StdResult<Vec<Vote>>>()?;

    Ok(ApplicationVotesResponse { votes })
}

pub fn query_ensure_active_nra(deps: Deps, address: Addr) -> StdResult<NraAccessResponse> {
    let cfg = CONFIG.load(deps.storage)?;

    // Check if sender is in bootstrap voters
    if cfg.bootstrap_voters.contains(&address) {
        return Ok(NraAccessResponse { allowed: true });
    }

    let allowed = match AGENTS.may_load(deps.storage, address)? {
        Some(agent) => agent.status == AgentStatus::Active,
        None => false,
    };

    Ok(NraAccessResponse { allowed })
}
