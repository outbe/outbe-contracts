use crate::state::{CREATOR, SEED};
use cosmwasm_schema::{cw_serde, QueryResponses};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Addr, Binary, Deps, Env, StdResult, Storage};
use cw_ownable::Ownership;

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(QuerySeedResponse)]
    Seed {},
    #[returns(cw_ownable::Ownership<String>)]
    GetCreatorOwnership {},
}

#[cw_serde]
pub struct QuerySeedResponse {
    pub seed: u64,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Seed {} => to_json_binary(&query_seed(deps.storage)?),
        QueryMsg::GetCreatorOwnership {} => to_json_binary(&query_creator_ownership(deps.storage)?),
    }
}

pub fn query_creator_ownership(storage: &dyn Storage) -> StdResult<Ownership<Addr>> {
    CREATOR.get_ownership(storage)
}

pub fn query_seed(storage: &dyn Storage) -> StdResult<QuerySeedResponse> {
    let seed = SEED.load(storage)?;
    Ok(QuerySeedResponse { seed })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::mock_dependencies;

    #[test]
    fn test_query_seed() {
        let mut deps = mock_dependencies();
        SEED.save(deps.as_mut().storage, &123u64).unwrap();

        let response = query_seed(&deps.storage).unwrap();
        assert_eq!(response.seed, 123u64);
    }
}
