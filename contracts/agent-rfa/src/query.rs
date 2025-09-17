use agent_common::query::{query_agent_by_address, query_all_agents};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{to_json_binary, Addr, Binary, Deps, Env, Order, StdResult};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // Agent
    #[returns(agent_common::msg::AgentResponse)]
    GetAgentByAddress { address: Addr },

    #[returns(agent_common::msg::ListAllAgentsResponse)]
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
