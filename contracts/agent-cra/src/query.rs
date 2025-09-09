use crate::state::{AGENTS};
use agent_nra::types::{
    Agent, AgentResponse,
    ListAllAgentsResponse,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{entry_point, to_json_binary, Addr, Binary, Deps, Env, Order, StdResult};
use cw_storage_plus::Bound;

pub const DEFAULT_LIMIT: u32 = 10;
pub const MAX_LIMIT: u32 = 1000;
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {

    // Agent
    #[returns(AgentResponse)]
    GetAgentByAddress { address: Addr },

    #[returns(ListAllAgentsResponse)]
    ListAllAgents {
        start_after: Option<Addr>,
        limit: Option<u32>,
        query_order: Option<Order>,
    },
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {

        QueryMsg::GetAgentByAddress { address } => {
            to_json_binary(&query_agent_by_address(deps, address)?)
        }
        QueryMsg::ListAllAgents {
            start_after,
            limit,
            query_order,
        } => to_json_binary(&query_all_agents(deps, start_after, limit, query_order)?),
    }
}



fn query_agent_by_address(deps: Deps, address: Addr) -> StdResult<AgentResponse> {
    let agent = AGENTS.load(deps.storage, address)?;

    Ok(AgentResponse { agent })
}

fn query_all_agents(
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
