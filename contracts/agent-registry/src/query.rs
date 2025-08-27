use crate::state::{ACCOUNTS, AGENTS, AGENT_VOTES};
use crate::types::{
    Account, AccountResponse, Agent, AgentResponse, AgentVotesResponse, ListAllAccountsResponse,
    ListAllResponse, Vote,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{entry_point, to_json_binary, Addr, Binary, Deps, Env, Order, StdResult};
use cw_storage_plus::Bound;

pub const DEFAULT_LIMIT: u32 = 10;
pub const MAX_LIMIT: u32 = 1000;
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ListAllResponse)]
    ListAll {
        start_after: Option<String>,
        limit: Option<u32>,
        query_order: Option<Order>,
    },
    #[returns(AgentResponse)]
    GetById { id: String },
    #[returns(ListAllResponse)]
    QueryByAddress {
        address: String,
        start_after: Option<String>,
        limit: Option<u32>,
        query_order: Option<Order>,
    },
    #[returns(AgentVotesResponse)]
    QueryVotesByAgent { id: String },

    #[returns(AccountResponse)]
    GetAccountByAddress { address: Addr },

    #[returns(ListAllAccountsResponse)]
    ListAllAccounts {
        start_after: Option<String>,
        limit: Option<u32>,
        query_order: Option<Order>,
    },
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ListAll {
            start_after,
            limit,
            query_order,
        } => to_json_binary(&query_all_agents(deps, start_after, limit, query_order)?),
        QueryMsg::GetById { id } => to_json_binary(&query_by_id(deps, id)?),
        QueryMsg::QueryByAddress {
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
        QueryMsg::QueryVotesByAgent { id } => to_json_binary(&query_votes_by_agent(deps, id)?),

        QueryMsg::GetAccountByAddress { address } => {
            to_json_binary(&query_account_by_address(deps, address)?)
        }
        QueryMsg::ListAllAccounts {
            start_after,
            limit,
            query_order,
        } => to_json_binary(&query_all_accounts(deps, start_after, limit, query_order)?),
    }
}

fn query_all_agents(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
    query_order: Option<Order>,
) -> StdResult<ListAllResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let order = query_order.unwrap_or(Order::Ascending);
    let (start, end) = match order {
        Order::Ascending => (start_after.as_deref().map(Bound::exclusive), None),
        Order::Descending => (None, start_after.as_deref().map(Bound::exclusive)),
    };

    let agents = AGENTS
        .range(deps.storage, start, end, order)
        .take(limit)
        .map(|item| item.map(|item| item.1))
        .collect::<StdResult<Vec<Agent>>>()?;

    Ok(ListAllResponse { agents })
}

fn query_by_id(deps: Deps, id: String) -> StdResult<AgentResponse> {
    let agent = AGENTS.load(deps.storage, id)?;

    Ok(AgentResponse { agent })
}

fn query_by_address(
    deps: Deps,
    address: String,
    start_after: Option<String>,
    limit: Option<u32>,
    query_order: Option<Order>,
) -> StdResult<ListAllResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let order = query_order.unwrap_or(Order::Ascending);
    let (start, end) = match order {
        Order::Ascending => (start_after.as_deref().map(Bound::exclusive), None),
        Order::Descending => (None, start_after.as_deref().map(Bound::exclusive)),
    };

    let addr = deps.api.addr_validate(&address)?;

    let agents = AGENTS
        .range(deps.storage, start, end, order)
        .filter_map(|item| match item {
            Ok((_id, agent)) if agent.wallet == addr => Some(Ok(agent)),
            Ok(_) => None,
            Err(e) => Some(Err(e)),
        })
        .take(limit)
        .collect::<StdResult<Vec<Agent>>>()?;

    Ok(ListAllResponse { agents })
}

pub fn query_votes_by_agent(deps: Deps, id: String) -> StdResult<AgentVotesResponse> {
    let votes: Vec<Vote> = AGENT_VOTES
        .prefix(id.as_str())
        .range(deps.storage, None, None, Order::Ascending)
        .map(|res| {
            let (_voter_addr, v) = res?;
            Ok(v)
        })
        .collect::<StdResult<Vec<Vote>>>()?;

    Ok(AgentVotesResponse { votes })
}

fn query_account_by_address(deps: Deps, address: Addr) -> StdResult<AccountResponse> {
    let account = ACCOUNTS.load(deps.storage, address)?;

    Ok(AccountResponse { account })
}

fn query_all_accounts(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
    query_order: Option<Order>,
) -> StdResult<ListAllAccountsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let order = query_order.unwrap_or(Order::Ascending);

    // Convert string to Addr if start_after is provided
    let start_bound = if let Some(addr_str) = start_after {
        let addr = deps.api.addr_validate(&addr_str)?;
        Some(Bound::exclusive(addr))
    } else {
        None
    };

    let (start, end) = match order {
        Order::Ascending => (start_bound, None),
        Order::Descending => (None, start_bound),
    };

    let accounts = ACCOUNTS
        .range(deps.storage, start, end, order)
        .take(limit)
        .map(|item| item.map(|(_addr, account)| account))
        .collect::<StdResult<Vec<Account>>>()?;

    Ok(ListAllAccountsResponse { accounts })
}
