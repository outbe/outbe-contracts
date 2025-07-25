use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, RandomResponse, SeedResponse};
use crate::state::RND;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, Event, MessageInfo, Response};

const CONTRACT_NAME: &str = "outbe.net:random-oracle";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    if msg.random_value.is_some() {
        RND.save(deps.storage, &msg.random_value.unwrap())?;
    }

    Ok(Response::default()
        .add_attribute("action", "random-oracle::instantiate")
        .add_event(Event::new("tribute::instantiate")))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetRandom { random_value } => {
            if random_value.is_some() {
                RND.save(deps.storage, &random_value.unwrap())?;
            } else {
                RND.remove(deps.storage);
            }
            Ok(Response::new()
                .add_attribute("action", "random-oracle::set_random")
                .add_event(Event::new("random-oracle::set_random")))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::RandomValue {
            from_range,
            to_range,
            count_values,
        } => {
            let value = get_random_value(deps, env, from_range, to_range, count_values)?;
            Ok(to_json_binary(&value)?)
        }
        QueryMsg::RandomSeed {} => {
            let value = get_seed(deps, env)?;
            Ok(to_json_binary(&value)?)
        }
    }
}

fn get_random_value(
    deps: Deps,
    env: Env,
    from_range: u64,
    to_range: u64,
    count_values: u64,
) -> Result<RandomResponse, ContractError> {
    if from_range >= to_range {
        return Err(ContractError::WrongInput {});
    }

    let stored_random = RND.may_load(deps.storage)?;
    let random_value = stored_random.unwrap_or(env.block.height);
    let range = to_range - from_range;

    let mut result: Vec<u64> = vec![];
    for i in 0..count_values {
        let value = (random_value + i) % range + from_range;
        result.push(value);
    }

    Ok(RandomResponse {
        random_values: result,
    })
}

fn get_seed(deps: Deps, env: Env) -> Result<SeedResponse, ContractError> {
    let stored_random = RND.may_load(deps.storage)?;
    let random_value = stored_random.unwrap_or(env.block.height);
    Ok(SeedResponse { seed: random_value })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};

    #[test]
    fn test_get_seed_with_stored_random() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let stored_value = 12345u64;

        RND.save(deps.as_mut().storage, &stored_value).unwrap();

        let result = get_seed(deps.as_ref(), env).unwrap();
        assert_eq!(result.seed, stored_value);
    }

    #[test]
    fn test_get_seed_without_stored_random() {
        let deps = mock_dependencies();
        let env = mock_env();

        let result = get_seed(deps.as_ref(), env.clone()).unwrap();
        assert_eq!(result.seed, env.block.height);
    }
}
