use crate::types::{TributeConfig, TributeData};
use cosmwasm_schema::{cw_serde, QueryResponses};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, Empty, Env, Order, StdResult};
use outbe_nft::state::Cw721Config;
use outbe_utils::date::WorldwideDay;

pub type TributeInfoResponse = outbe_nft::msg::NftInfoResponse<TributeData>;
pub type TributeContractInfoResponse = outbe_nft::msg::ContractInfoResponse<TributeConfig>;

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(TributeContractInfoResponse)]
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

    #[returns(TributeInfoResponse)]
    NftInfo { token_id: String },

    /// Returns all tokens owned by the given address.
    /// Same as `AllTokens` but with owner filter.
    #[returns(outbe_nft::msg::TokensResponse)]
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
        query_order: Option<Order>,
    },
    /// With Enumerable extension.
    /// Requires pagination. Lists all token_ids controlled by the contract.
    #[returns(outbe_nft::msg::TokensResponse)]
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
        query_order: Option<Order>,
    },

    /// Returns all tokens created in the given date with an optional filter by status.
    #[returns(DailyTributesResponse)]
    DailyTributes { date: WorldwideDay },
}

#[cw_serde]
pub struct FullTributeData {
    pub token_id: String,
    pub owner: String,
    pub data: TributeData,
}

#[cw_serde]
pub struct DailyTributesResponse {
    pub tributes: Vec<FullTributeData>,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ContractInfo {} => to_json_binary(&outbe_nft::query::query_contract_info::<
            TributeConfig,
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
            TributeData,
        >(deps.storage, token_id)?),
        QueryMsg::Tokens {
            owner,
            start_after,
            limit,
            query_order,
        } => to_json_binary(&outbe_nft::query::query_tokens(
            deps,
            &env,
            owner,
            start_after,
            limit,
            query_order,
        )?),
        QueryMsg::AllTokens {
            start_after,
            limit,
            query_order,
        } => to_json_binary(&outbe_nft::query::query_all_tokens(
            deps,
            &env,
            start_after,
            limit,
            query_order,
        )?),
        QueryMsg::DailyTributes { date } => {
            to_json_binary(&query_daily_tributes(deps, &env, date)?)
        }
    }
}
fn query_daily_tributes(
    deps: Deps,
    _env: &Env,
    date: WorldwideDay,
) -> StdResult<DailyTributesResponse> {
    let tokens: StdResult<Vec<FullTributeData>> =
        Cw721Config::<TributeData, Option<Empty>>::default()
            .nft_info
            .range(deps.storage, None, None, Order::Ascending)
            .filter_map(|item| match item {
                Ok((id, tribute)) if tribute.extension.worldwide_day == date => {
                    Some(Ok(FullTributeData {
                        token_id: id,
                        owner: tribute.owner.to_string(),
                        data: tribute.extension,
                    }))
                }
                _ => None,
            })
            .collect();

    Ok(DailyTributesResponse { tributes: tokens? })
}

#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate};
    use crate::msg::{InstantiateMsg, TributeCollectionExtension};
    use crate::query::{query, QueryMsg};
    use cosmwasm_std::{Addr, Decimal};
    use cw_multi_test::{App, ContractWrapper, Executor};
    use cw_ownable::Ownership;
    use outbe_utils::denom::Denom;
    use std::str::FromStr;

    #[test]
    fn test_query_config() {
        let mut app = App::default();
        let owner = app.api().addr_make("owner");

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let init_msg = InstantiateMsg {
            name: "tribute".to_string(),
            symbol: "t".to_string(),
            collection_info_extension: TributeCollectionExtension {
                symbolic_rate: Decimal::from_str("0.08").unwrap(),
                native_token: Denom::Native("native".to_string()),
                price_oracle: Addr::unchecked("price_oracle"),
            },
            minter: None,
            burner: None,
            creator: None,
        };

        let contract_addr = app
            .instantiate_contract(code_id, owner.clone(), &init_msg, &[], "t1", None)
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
