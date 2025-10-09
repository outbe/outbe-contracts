use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use crate::state::CREATOR;
use cosmwasm_schema::cw_serde;
use std::str::FromStr;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, DepsMut, Env, Event, MessageInfo, Response, Uint128};
use cw2::set_contract_version;
use outbe_utils::date::WorldwideDay;

const CONTRACT_NAME: &str = "outbe.net:token-allocator";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // use info.sender if None is passed
    let creator: &str = match msg.creator.as_deref() {
        Some(creator) => creator,
        None => info.sender.as_str(),
    };

    CREATOR.initialize_owner(deps.storage, deps.api, Some(creator))?;

    Ok(Response::default()
        .add_attribute("action", "token-allocator::instantiate")
        .add_event(Event::new("token-allocator::instantiate").add_attribute("creator", creator)))
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
    match msg {
        ExecuteMsg::AllocateTokens { date } => execute_allocate_tokens(deps, env, info, date),
    }
}

#[cw_serde]
pub struct AllocationResult {
    pub day: WorldwideDay,
    pub allocation: Uint128,
}

fn execute_allocate_tokens(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    date: WorldwideDay,
) -> Result<Response, ContractError> {
    // let daily_total_allocation = crate::query::query_daily_allocation(env)?;
    let daily_total_allocation = mock_daily_allocation();

    let allocation_data = AllocationResult {
        day: date,
        allocation: daily_total_allocation,
    };
    let allocation_data = to_json_binary(&allocation_data)?;

    Ok(Response::default()
        .set_data(allocation_data)
        .add_attribute("action", "token-allocator::allocate_tokens")
        .add_attribute("date", date.to_string())
        .add_attribute("daily_total_allocation", daily_total_allocation.to_string()))
}

pub fn mock_daily_allocation() -> Uint128 {
    Uint128::from_str("1414599614792365000000000000").unwrap()
}

#[cfg(test)]
mod tests {
    use crate::contract::mock_daily_allocation;

    #[test]
    fn mock_daily_allocation_ok() {
        println!("mock_daily_allocation_ok = {}", mock_daily_allocation());
    }
}
