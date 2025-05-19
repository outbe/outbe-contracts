use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{entry_point, Binary, Deps, Env, StdResult};

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    Ok(Binary::default())
}
