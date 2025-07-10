use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdResult};
use cw20_base::contract::query as cw20_query;
use cw20_base::msg::QueryMsg as Cw20QueryMsg;

use crate::msg::{CheckTicketResponse, QueryMsg};
use crate::state::TICKETS;

pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::CheckTicket { ticket } => to_json_binary(&query_check_ticket(deps, ticket)?),
        QueryMsg::Balance { address } => {
            let cw20_msg = Cw20QueryMsg::Balance { address };
            cw20_query(deps, env, cw20_msg)
        }
        QueryMsg::TokenInfo {} => {
            let cw20_msg = Cw20QueryMsg::TokenInfo {};
            cw20_query(deps, env, cw20_msg)
        }
        QueryMsg::Minter {} => {
            let cw20_msg = Cw20QueryMsg::Minter {};
            cw20_query(deps, env, cw20_msg)
        }
        QueryMsg::AllAccounts { start_after, limit } => {
            let cw20_msg = Cw20QueryMsg::AllAccounts { start_after, limit };
            cw20_query(deps, env, cw20_msg)
        }
    }
}

fn query_check_ticket(deps: Deps, ticket: String) -> StdResult<CheckTicketResponse> {
    let exists = TICKETS.may_load(deps.storage, ticket)?.unwrap_or(false);
    Ok(CheckTicketResponse { exists })
}