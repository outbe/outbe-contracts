use crate::types::{NodConfig, NodData};
use cosmwasm_schema::{cw_serde, QueryResponses};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, Order, StdResult};
use cw_ownable::Ownership;
use outbe_nft::msg::{
    ContractInfoResponse, NftInfoResponse, NumTokensResponse, OwnerOfResponse, TokensResponse,
};

/// Query messages for Nod contract
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ContractInfoResponse<NodConfig>)]
    ContractInfo {},

    #[returns(OwnerOfResponse)]
    OwnerOf { token_id: String },

    #[returns(NumTokensResponse)]
    NumTokens {},

    #[returns(Ownership<String>)]
    GetMinterOwnership {},

    #[returns(Ownership<String>)]
    GetCreatorOwnership {},

    #[returns(NftInfoResponse<NodData>)]
    NftInfo { token_id: String },

    #[returns(TokensResponse)]
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
        query_order: Option<Order>,
    },

    #[returns(TokensResponse)]
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
        query_order: Option<Order>,
    },
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ContractInfo {} => to_json_binary(&outbe_nft::query::query_contract_info::<
            NodConfig,
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
            NodData,
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{execute, instantiate};
    use crate::msg::{
        ExecuteMsg, InstantiateMsg, NodCollectionExtension, NodEntity, SubmitExtension,
    };
    use crate::types::{NodData, State};
    use cosmwasm_std::{Decimal, Timestamp, Uint128};
    use cw20::Denom;
    use cw_multi_test::{App, ContractWrapper, Executor};
    use std::str::FromStr;

    #[test]
    fn test_instantiate_submit_query_and_burn() {
        let mut app = App::default();
        let owner = app.api().addr_make("owner");

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let init_msg = InstantiateMsg {
            name: "nod".to_string(),
            symbol: "NOD".to_string(),
            collection_info_extension: NodCollectionExtension {},
            minter: None,
            creator: None,
            burner: None,
        };
        let contract_addr = app
            .instantiate_contract(code_id, owner.clone(), &init_msg, &[], "nod1", None)
            .unwrap();

        // initially no tokens
        let resp: outbe_nft::msg::NumTokensResponse = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::NumTokens {})
            .unwrap();
        assert_eq!(resp.count, 0);

        // default minter/creator is owner
        let resp: cw_ownable::Ownership<String> = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetMinterOwnership {})
            .unwrap();
        assert_eq!(resp.owner.unwrap(), owner.to_string());
        let resp: cw_ownable::Ownership<String> = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetCreatorOwnership {})
            .unwrap();
        assert_eq!(resp.owner.unwrap(), owner.to_string());

        // Submit (mint) a new Nod NFT
        let token_id = "token1".to_string();
        let recipient = app.api().addr_make("recipient");
        let entity = NodEntity {
            nod_id: "nod123".to_string(),
            settlement_token: Denom::Native("uset".to_string()),
            symbolic_rate: Decimal::from_str("1.23").unwrap(),
            nominal_minor_rate: Uint128::new(10),
            issuance_minor_rate: Decimal::from_str("20").unwrap(),
            symbolic_minor_load: Uint128::new(30),
            vector_minor_rate: Uint128::new(40),
            floor_minor_price: Decimal::from_str("50").unwrap(),
            state: State::Issued,
            address: recipient.to_string(),
        };
        let submit_ext = SubmitExtension {
            entity: entity.clone(),
            created_at: Some(Timestamp::from_seconds(12345)),
        };
        let exec_msg = ExecuteMsg::Submit {
            token_id: token_id.clone(),
            owner: recipient.to_string(),
            extension: Box::new(submit_ext.clone()),
        };
        app.execute_contract(owner.clone(), contract_addr.clone(), &exec_msg, &[])
            .unwrap();

        // after mint, num tokens = 1
        let resp: outbe_nft::msg::NumTokensResponse = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::NumTokens {})
            .unwrap();
        assert_eq!(resp.count, 1);

        // OwnerOf should return recipient
        let resp: outbe_nft::msg::OwnerOfResponse = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &QueryMsg::OwnerOf {
                    token_id: token_id.clone(),
                },
            )
            .unwrap();
        assert_eq!(resp.owner, recipient.to_string());

        // NftInfo should return the correct extension
        let resp: outbe_nft::msg::NftInfoResponse<NodData> = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &QueryMsg::NftInfo {
                    token_id: token_id.clone(),
                },
            )
            .unwrap();
        assert_eq!(resp.extension.nod_id, entity.nod_id);
        assert_eq!(resp.extension.settlement_token, entity.settlement_token);
        assert_eq!(resp.extension.symbolic_rate, entity.symbolic_rate);
        assert_eq!(resp.extension.nominal_minor_rate, entity.nominal_minor_rate);
        assert_eq!(
            resp.extension.issuance_minor_rate,
            entity.issuance_minor_rate
        );
        assert_eq!(
            resp.extension.symbolic_minor_load,
            entity.symbolic_minor_load
        );
        assert_eq!(resp.extension.vector_minor_rate, entity.vector_minor_rate);
        assert_eq!(resp.extension.floor_minor_price, entity.floor_minor_price);
        assert_eq!(resp.extension.state, entity.state);
        assert_eq!(resp.extension.address, entity.address);
        assert_eq!(resp.extension.created_at, submit_ext.created_at.unwrap());

        // Tokens for recipient should include token_id
        let resp: outbe_nft::msg::TokensResponse = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &QueryMsg::Tokens {
                    owner: recipient.to_string(),
                    start_after: None,
                    limit: None,
                    query_order: None,
                },
            )
            .unwrap();
        assert_eq!(resp.tokens, vec![token_id.clone()]);

        // AllTokens should include token_id
        let resp: outbe_nft::msg::TokensResponse = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &QueryMsg::AllTokens {
                    start_after: None,
                    limit: None,
                    query_order: None,
                },
            )
            .unwrap();
        assert_eq!(resp.tokens, vec![token_id.clone()]);

        // Burn the token
        let exec_msg = ExecuteMsg::Burn {
            token_id: token_id.clone(),
        };
        app.execute_contract(owner.clone(), contract_addr.clone(), &exec_msg, &[])
            .unwrap();

        // after burn, num tokens = 0
        let resp: outbe_nft::msg::NumTokensResponse = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::NumTokens {})
            .unwrap();
        assert_eq!(resp.count, 0);
    }
}
