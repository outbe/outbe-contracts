use cosmwasm_std::{Addr, Deps, Order, StdResult};
use cw_storage_plus::Bound;
use crate::msg::{AgentResponse, ListAllAgentsResponse};
use crate::state::AGENTS;
use crate::types::Agent;


pub const DEFAULT_LIMIT: u32 = 10;
pub const MAX_LIMIT: u32 = 1000;
pub fn query_agent_by_address(deps: Deps, address: Addr) -> StdResult<AgentResponse> {
    let agent = AGENTS.load(deps.storage, address)?;

    Ok(AgentResponse { agent })
}

pub fn query_all_agents(
    deps: Deps,
    start_after: Option<Addr>,
    limit: Option<u32>,
    query_order: Option<Order>,
) -> StdResult<ListAllAgentsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let order = query_order.unwrap_or(Order::Ascending);

    let (start, end) = match order {
        Order::Ascending => (start_after.map(Bound::exclusive), None),
        Order::Descending => (None, start_after.map(Bound::exclusive)),
    };

    let agents = AGENTS
        .range(deps.storage, start, end, order)
        .take(limit)
        .map(|item| item.map(|(_addr, account)| account))
        .collect::<StdResult<Vec<Agent>>>()?;

    Ok(ListAllAgentsResponse { agents })
}
