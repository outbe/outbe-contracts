use crate::types::{CUConfig, ConsumptionUnitData};
use cosmwasm_schema::{cw_serde, QueryResponses};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdResult};

pub type ConsumptionUnitInfoResponse = outbe_nft::msg::NftInfoResponse<ConsumptionUnitData>;

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(outbe_nft::msg::ContractInfoResponse<CUConfig>)]
    ContractInfo {},

    // TODO add Cw721 config as well
    #[returns(outbe_nft::msg::OwnerOfResponse)]
    OwnerOf { token_id: String },

    #[returns(outbe_nft::msg::NumTokensResponse)]
    NumTokens {},

    #[returns(cw_ownable::Ownership<String>)]
    GetMinterOwnership {},

    #[returns(cw_ownable::Ownership<String>)]
    GetCreatorOwnership {},

    #[returns(ConsumptionUnitInfoResponse)]
    NftInfo { token_id: String },

    /// Returns all tokens owned by the given address.
    /// Same as `AllTokens` but with owner filter.
    #[returns(outbe_nft::msg::TokensResponse)]
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// With Enumerable extension.
    /// Requires pagination. Lists all token_ids controlled by the contract.
    #[returns(outbe_nft::msg::TokensResponse)]
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ContractInfo {} => to_json_binary(&outbe_nft::query::query_contract_info::<
            CUConfig,
        >(deps.storage)?),
        QueryMsg::OwnerOf { token_id } => to_json_binary(&outbe_nft::query::query_owner_of(
            deps.storage,
            &env,
            token_id,
        )?),
        QueryMsg::NumTokens {} => {
            to_json_binary(&outbe_nft::query::query_num_tokens(deps.storage)?)
        }
        QueryMsg::GetMinterOwnership {} => {
            to_json_binary(&outbe_nft::query::query_minter_ownership(deps.storage)?)
        }
        QueryMsg::GetCreatorOwnership {} => {
            to_json_binary(&outbe_nft::query::query_creator_ownership(deps.storage)?)
        }
        QueryMsg::NftInfo { token_id } => to_json_binary(&outbe_nft::query::query_nft_info::<
            ConsumptionUnitData,
        >(deps.storage, token_id)?),
        QueryMsg::Tokens {
            owner,
            start_after,
            limit,
        } => to_json_binary(&outbe_nft::query::query_tokens(
            deps,
            &env,
            owner,
            start_after,
            limit,
        )?),
        QueryMsg::AllTokens { start_after, limit } => to_json_binary(
            &outbe_nft::query::query_all_tokens(deps, &env, start_after, limit)?,
        ),
    }
}

// Query

#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate};
    use crate::msg::{ConsumptionUnitCollectionExtension, InstantiateMsg};
    use crate::query::{query, QueryMsg};
    use cosmwasm_std::Addr;
    use cw20::Denom;
    use cw_multi_test::{App, ContractWrapper, Executor};
    use cw_ownable::Ownership;

    #[test]
    fn test_query_config() {
        let mut app = App::default();
        let owner = app.api().addr_make("owner");

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let init_msg = InstantiateMsg {
            name: "consumption unit".to_string(),
            symbol: "cu".to_string(),
            collection_info_extension: ConsumptionUnitCollectionExtension {
                settlement_token: Denom::Cw20(Addr::unchecked("settlement")),
                native_token: Denom::Native("native".to_string()),
                price_oracle: Addr::unchecked("price_oracle"),
            },
            minter: None,
            creator: None,
        };

        let contract_addr = app
            .instantiate_contract(code_id, owner.clone(), &init_msg, &[], "cu1", None)
            .unwrap();

        let response: outbe_nft::msg::NumTokensResponse = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::NumTokens {})
            .unwrap();
        assert_eq!(response.count, 0);

        let response: Ownership<String> = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetMinterOwnership {})
            .unwrap();

        assert_eq!(response.owner.unwrap(), owner.to_string());

        let response: Ownership<String> = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetCreatorOwnership {})
            .unwrap();

        assert_eq!(response.owner.unwrap(), owner.to_string());
    }
}
