use cosmwasm_std::{Addr, Deps, Empty, Env, Order, StdResult, Storage};
use cw_ownable::Ownership;
use cw_storage_plus::Bound;

use crate::msg::ContractInfoResponse;
use crate::traits::Cw721CollectionConfig;
use crate::{
    msg::{NftInfoResponse, NumTokensResponse, OwnerOfResponse, TokensResponse},
    state::{Cw721Config, CREATOR, MINTER},
    traits::Cw721State,
};

pub const DEFAULT_LIMIT: u32 = 10;
pub const MAX_LIMIT: u32 = 1000;

// --- query helpers ---
pub fn query_minter_ownership(storage: &dyn Storage) -> StdResult<Ownership<Addr>> {
    MINTER.get_ownership(storage)
}

pub fn query_creator_ownership(storage: &dyn Storage) -> StdResult<Ownership<Addr>> {
    CREATOR.get_ownership(storage)
}

pub fn query_num_tokens(storage: &dyn Storage) -> StdResult<NumTokensResponse> {
    let count = Cw721Config::<Option<Empty>, Option<Empty>>::default().token_count(storage)?;
    Ok(NumTokensResponse { count })
}

pub fn query_nft_info<TNftExtension>(
    storage: &dyn Storage,
    token_id: String,
) -> StdResult<NftInfoResponse<TNftExtension>>
where
    TNftExtension: Cw721State,
{
    let info = Cw721Config::<TNftExtension, Option<Empty>>::default()
        .nft_info
        .load(storage, &token_id)?;
    Ok(NftInfoResponse {
        extension: info.extension,
        owner: info.owner,
        token_id,
    })
}

pub fn query_contract_info<TCollectionConfig>(
    storage: &dyn Storage,
) -> StdResult<ContractInfoResponse<TCollectionConfig>>
where
    TCollectionConfig: Cw721CollectionConfig,
{
    let info = Cw721Config::<Option<Empty>, TCollectionConfig>::default()
        .collection_info
        .load(storage)?;
    let config = Cw721Config::<Option<Empty>, TCollectionConfig>::default()
        .collection_config
        .load(storage)?;

    Ok(ContractInfoResponse {
        collection_info: info,
        collection_config: config,
    })
}

pub fn query_owner_of(
    storage: &dyn Storage,
    _env: &Env,
    token_id: String,
) -> StdResult<OwnerOfResponse> {
    let nft_info = Cw721Config::<Option<Empty>, Option<Empty>>::default()
        .nft_info
        .load(storage, &token_id)?;
    Ok(OwnerOfResponse {
        owner: nft_info.owner.to_string(),
    })
}

pub fn query_tokens(
    deps: Deps,
    _env: &Env,
    owner: String,
    start_after: Option<String>,
    limit: Option<u32>,
    query_order: Option<Order>,
) -> StdResult<TokensResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let order = query_order.unwrap_or(Order::Ascending);
    let (start, end) = match order {
        Order::Ascending => (start_after.as_deref().map(Bound::exclusive), None),
        Order::Descending => (None, start_after.as_deref().map(Bound::exclusive)),
    };
    let owner_addr = deps.api.addr_validate(&owner)?;
    let tokens: Vec<String> = Cw721Config::<Option<Empty>, Option<Empty>>::default()
        .nft_info
        .idx
        .owner
        .prefix(owner_addr)
        .keys(deps.storage, start, end, order)
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?;

    Ok(TokensResponse { tokens })
}

pub fn query_all_tokens(
    deps: Deps,
    _env: &Env,
    start_after: Option<String>,
    limit: Option<u32>,
    query_order: Option<Order>,
) -> StdResult<TokensResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let order = query_order.unwrap_or(Order::Ascending);

    let (start, end) = match order {
        Order::Ascending => (start_after.as_deref().map(Bound::exclusive), None),
        Order::Descending => (None, start_after.as_deref().map(Bound::exclusive)),
    };
    let tokens: StdResult<Vec<String>> = Cw721Config::<Option<Empty>, Option<Empty>>::default()
        .nft_info
        .range(deps.storage, start, end, order)
        .take(limit)
        .map(|item| item.map(|(k, _)| k))
        .collect();

    Ok(TokensResponse { tokens: tokens? })
}

#[cfg(test)]
mod tests {

    use crate::query::query_all_tokens;
    use crate::state::{Cw721Config, NftInfo};
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::Order;
    use cosmwasm_std::{Addr, Empty};

    #[test]
    fn test_query_all_tokens_asc() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let config = Cw721Config::<Option<Empty>, Option<Empty>>::default();

        for i in 0..10 {
            config
                .nft_info
                .save(
                    &mut deps.storage,
                    i.to_string().as_str(),
                    &NftInfo {
                        owner: Addr::unchecked("me"),
                        token_uri: None,
                        extension: None,
                    },
                )
                .unwrap();
        }

        assert!(!config.nft_info.is_empty(&deps.storage));

        // Ascending
        let response =
            query_all_tokens(deps.as_ref(), &env, None, Some(10), Some(Order::Ascending)).unwrap();
        assert_eq!(response.tokens.len(), 10);
        assert_eq!(response.tokens.first().unwrap(), "0");
        assert_eq!(response.tokens.last().unwrap(), "9");
        // Ascending, start_after = "2", limit 3
        let response = query_all_tokens(
            deps.as_ref(),
            &env,
            Some("2".to_string()),
            Some(3),
            Some(Order::Ascending),
        )
        .unwrap();
        println!("res {:?}", response.tokens);

        assert_eq!(response.tokens.len(), 3);
        assert_eq!(response.tokens.first().unwrap(), "3");
        assert_eq!(response.tokens.last().unwrap(), "5");
    }

    #[test]
    fn test_query_all_tokens_desc() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let config = Cw721Config::<Option<Empty>, Option<Empty>>::default();

        for i in 0..10 {
            config
                .nft_info
                .save(
                    &mut deps.storage,
                    i.to_string().as_str(),
                    &NftInfo {
                        owner: Addr::unchecked("me"),
                        token_uri: None,
                        extension: None,
                    },
                )
                .unwrap();
        }

        assert!(!config.nft_info.is_empty(&deps.storage));

        // Descending
        let response =
            query_all_tokens(deps.as_ref(), &env, None, Some(10), Some(Order::Descending)).unwrap();
        assert_eq!(response.tokens.len(), 10);
        assert_eq!(response.tokens.first().unwrap(), "9");
        assert_eq!(response.tokens.last().unwrap(), "0");
        // Descending, start_after = "7", limit 3
        let response = query_all_tokens(
            deps.as_ref(),
            &env,
            Some("7".to_string()),
            Some(3),
            Some(Order::Descending),
        )
        .unwrap();
        println!("res {:?}", response.tokens);

        assert_eq!(response.tokens.len(), 3);
        assert_eq!(response.tokens.first().unwrap(), "6");
        assert_eq!(response.tokens.last().unwrap(), "4");
    }
}
