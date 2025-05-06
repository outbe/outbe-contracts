use crate::state::{CONFIG, CREATOR};
use crate::types::CommitmentTier;
use cosmwasm_schema::{cw_serde, QueryResponses};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Addr, Binary, Deps, Env, StdResult, Storage};
use cw_ownable::Ownership;

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns all tiers
    #[returns(AllTiersResponse)]
    Tiers {},

    #[returns(cw_ownable::Ownership<String>)]
    GetCreatorOwnership {},
}

#[cw_serde]
pub struct AllTiersResponse {
    pub tiers: Vec<CommitmentTier>,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Tiers {} => to_json_binary(&query_tiers(deps.storage)?),
        QueryMsg::GetCreatorOwnership {} => to_json_binary(&query_creator_ownership(deps.storage)?),
    }
}

// Query
pub fn query_tiers(storage: &dyn Storage) -> StdResult<AllTiersResponse> {
    let config = CONFIG.load(storage)?;
    Ok(AllTiersResponse {
        tiers: config.tiers,
    })
}

pub fn query_creator_ownership(storage: &dyn Storage) -> StdResult<Ownership<Addr>> {
    CREATOR.get_ownership(storage)
}

#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate};
    use crate::msg::InstantiateMsg;
    use crate::query::{query, AllTiersResponse, QueryMsg};
    use cosmwasm_std::Decimal;
    use cw_multi_test::{App, ContractWrapper, Executor};
    use std::str::FromStr;

    #[test]
    fn test_query_tiers() {
        let mut app = App::default();
        let owner = app.api().addr_make("owner");

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let init_msg = InstantiateMsg {
            tiers: None,
            creator: None,
        };

        let contract_addr = app
            .instantiate_contract(code_id, owner.clone(), &init_msg, &[], "tiers1", None)
            .unwrap();

        let response: AllTiersResponse = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::Tiers {})
            .unwrap();
        assert_eq!(response.tiers.len(), 16);
        assert_eq!(response.tiers.first().unwrap().tier_id, 1);
        assert_eq!(
            response.tiers.first().unwrap().weight,
            Decimal::from_str("0.125").unwrap()
        );
        assert_eq!(response.tiers.last().unwrap().tier_id, 16);
        assert_eq!(
            response.tiers.last().unwrap().weight,
            Decimal::from_str("1").unwrap()
        );
    }
}
