use cosmwasm_std::{Binary, Deps, Env, StdResult};

use crate::msg::{QueryMsg};

pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
    }
}

