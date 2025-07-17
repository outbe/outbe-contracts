use cosmwasm_std::{to_json_binary, Binary, Deps, Env, Order, StdResult};

use crate::msg::{
    AccessListResponse, AccessPermissionsResponse, CanMintResponse, ConfigResponse, QueryMsg,
};
use crate::state::{TokenType, ACCESS_LIST, CONFIG};

/// Maximum number of addresses to return in a single query
const MAX_LIMIT: u32 = 100;
/// Default number of addresses to return if no limit specified
const DEFAULT_LIMIT: u32 = 30;

/// Main query dispatcher
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::AccessPermissions { address } => {
            to_json_binary(&query_access_permissions(deps, address)?)
        }
        QueryMsg::AccessList { start_after, limit } => {
            to_json_binary(&query_access_list(deps, start_after, limit)?)
        }
        QueryMsg::CanMint {
            address,
            token_type,
        } => to_json_binary(&query_can_mint(deps, address, token_type)?),
    }
}

/// Query contract configuration
fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

/// Query access permissions for a specific address
fn query_access_permissions(deps: Deps, address: String) -> StdResult<AccessPermissionsResponse> {
    let addr = deps.api.addr_validate(&address)?;
    let permissions = ACCESS_LIST.may_load(deps.storage, &addr)?;

    Ok(AccessPermissionsResponse {
        address: addr,
        permissions,
    })
}

/// Query access list with optional pagination
fn query_access_list(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<AccessListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let mut addresses = Vec::new();
    let mut iter = ACCESS_LIST.range(deps.storage, None, None, Order::Ascending);

    let has_start_after = start_after.is_some();

    // Skip to start_after if provided
    if let Some(start_addr) = start_after {
        let start_validated = deps.api.addr_validate(&start_addr)?;
        for item in iter.by_ref() {
            let (addr, _) = item?;
            if addr > start_validated {
                addresses.push((addr.clone(), ACCESS_LIST.load(deps.storage, &addr)?));
                break;
            }
        }
    }

    // Collect remaining items up to limit
    for item in iter.take(if has_start_after { limit - 1 } else { limit }) {
        let (addr, permissions) = item?;
        addresses.push((addr, permissions));
    }

    Ok(AccessListResponse { addresses })
}

/// Query if an address can mint a specific token type
fn query_can_mint(
    deps: Deps,
    address: String,
    token_type: TokenType,
) -> StdResult<CanMintResponse> {
    let addr = deps.api.addr_validate(&address)?;

    match ACCESS_LIST.may_load(deps.storage, &addr)? {
        Some(permissions) => {
            let can_mint = match token_type {
                TokenType::Gratis => permissions.can_mint_gratis,
                TokenType::Promis => permissions.can_mint_promis,
            };

            let reason = if !can_mint {
                Some(format!(
                    "Address does not have permission to mint {:?} tokens",
                    token_type
                ))
            } else {
                None
            };

            Ok(CanMintResponse { can_mint, reason })
        }
        None => Ok(CanMintResponse {
            can_mint: false,
            reason: Some("Address not found in access list".to_string()),
        }),
    }
}
