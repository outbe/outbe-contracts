use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, QueryRequest, Response,
    StdResult, Uint128, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;
use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{AccessPermissions, Config, TokenType, ACCESS_LIST, CONFIG};

// Import types from other contracts
use outbe_nft::msg::NftInfoResponse;

// Import types from nod and price-oracle libraries
use nod::msg::ExecuteMsg as NodExecuteMsg;
use nod::query::QueryMsg as NodQueryMsg;
use nod::types::{NodData, State as NodState};
use price_oracle::query::QueryMsg as PriceOracleQueryMsg;
use price_oracle::types::TokenPairPrice;

/// Contract name and version for migration info
pub const CONTRACT_NAME: &str = "outbe.net:token-minter";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Contract instantiation entry point
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Validate and store contract addresses
    let gratis_contract = deps.api.addr_validate(&msg.gratis_contract)?;
    let promis_contract = deps.api.addr_validate(&msg.promis_contract)?;
    let price_oracle_contract = deps.api.addr_validate(&msg.price_oracle_contract)?;
    let nod_contract = deps.api.addr_validate(&msg.nod_contract)?;

    // Create configuration with the instantiator as admin
    let config = Config {
        admin: info.sender.clone(),
        gratis_contract,
        promis_contract,
        price_oracle_contract,
        nod_contract,
    };
    CONFIG.save(deps.storage, &config)?;

    // Add the admin to the access list with full permissions
    let admin_permissions = AccessPermissions {
        can_mint_gratis: true,
        can_mint_promis: true,
        note: Some("Contract admin".to_string()),
    };
    ACCESS_LIST.save(deps.storage, &info.sender, &admin_permissions)?;
    for access in msg.access_list {
        let access_addr = deps.api.addr_validate(&access.address)?;
        ACCESS_LIST.save(deps.storage, &access_addr, &access.permissions)?;
    }

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", info.sender)
        .add_attribute("gratis_contract", msg.gratis_contract)
        .add_attribute("promis_contract", msg.promis_contract)
        .add_attribute("price_oracle_contract", msg.price_oracle_contract)
        .add_attribute("nod_contract", msg.nod_contract))
}

/// Contract execution entry point
#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mine {
            recipient,
            amount,
            token_type,
        } => execute_mine(deps, env, info, recipient, amount, token_type),
        ExecuteMsg::MineGratisWithNod { nod_token_id } => {
            execute_mine_gratis_with_nod(deps, env, info, nod_token_id)
        }
        ExecuteMsg::AddToAccessList {
            address,
            permissions,
        } => execute_add_to_access_list(deps, info, address, permissions),
        ExecuteMsg::RemoveFromAccessList { address } => {
            execute_remove_from_access_list(deps, info, address)
        }
        ExecuteMsg::UpdatePermissions {
            address,
            permissions,
        } => execute_update_permissions(deps, info, address, permissions),
        ExecuteMsg::TransferAdmin { new_admin } => execute_transfer_admin(deps, info, new_admin),
        ExecuteMsg::UpdateContracts {
            gratis_contract,
            promis_contract,
            price_oracle_contract,
            nod_contract,
        } => execute_update_contracts(
            deps,
            info,
            gratis_contract,
            promis_contract,
            price_oracle_contract,
            nod_contract,
        ),
    }
}

/// Execute mint function - mints tokens by calling the appropriate token contract
pub fn execute_mine(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
    token_type: TokenType,
) -> Result<Response, ContractError> {
    // Validate amount
    if amount.is_zero() {
        return Err(ContractError::InvalidAmount {});
    }

    // Validate recipient address
    let _recipient_addr = deps.api.addr_validate(&recipient)?;

    // Check if sender has permission to mint the requested token type
    let permissions = ACCESS_LIST
        .may_load(deps.storage, &info.sender)?
        .ok_or(ContractError::AddressNotInAccessList {})?;

    let can_mint = match token_type {
        TokenType::Gratis => permissions.can_mint_gratis,
        TokenType::Promis => permissions.can_mint_promis,
    };

    if !can_mint {
        return Err(ContractError::NoMintPermission {
            token_type: format!("{:?}", token_type),
        });
    }

    // Get contract configuration
    let config = CONFIG.load(deps.storage)?;

    // Determine target contract address
    let target_contract = match token_type {
        TokenType::Gratis => config.gratis_contract,
        TokenType::Promis => config.promis_contract,
    };

    // Create mint message for the target token contract
    let mint_msg = Cw20ExecuteMsg::Mint {
        recipient: recipient.clone(),
        amount,
    };

    let wasm_msg = WasmMsg::Execute {
        contract_addr: target_contract.to_string(),
        msg: to_json_binary(&mint_msg)?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(wasm_msg)
        .add_attribute("method", "mint")
        .add_attribute("minter", info.sender)
        .add_attribute("recipient", recipient)
        .add_attribute("amount", amount)
        .add_attribute("token_type", format!("{:?}", token_type))
        .add_attribute("target_contract", target_contract))
}

/// Execute mine gratis with nod - mines Gratis tokens using a qualified Nod NFT
/// This function checks if the current price from Price Oracle is >= floor_price_minor
/// If qualified, it will mint Gratis tokens based on gratis_load_minor and burn the Nod NFT
pub fn execute_mine_gratis_with_nod(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    nod_token_id: String,
) -> Result<Response, ContractError> {
    // Get contract configuration
    let config = CONFIG.load(deps.storage)?;

    // Query the Nod NFT to get its data
    let nod_info_query = QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: config.nod_contract.to_string(),
        msg: to_json_binary(&NodQueryMsg::NftInfo {
            token_id: nod_token_id.clone(),
        })?,
    });

    let nod_info_response: NftInfoResponse<NodData> = deps.querier.query(&nod_info_query)?;
    let nod_data = nod_info_response.extension;

    // Check if the sender is the owner of the Nod NFT (entitled to mine Gratis)
    if info.sender.as_str() != nod_data.owner {
        return Err(ContractError::NotNodOwner {});
    }

    // Check if the Nod is in Issued state (can only mine from Issued state)
    if nod_data.state != NodState::Issued {
        return Err(ContractError::NodNotIssued {});
    }

    // Query the Price Oracle to get the current price
    let price_query = QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: config.price_oracle_contract.to_string(),
        msg: to_json_binary(&PriceOracleQueryMsg::GetPrice {})?,
    });

    let price_response: TokenPairPrice = deps.querier.query(&price_query)?;

    // Check if current price is >= floor price (Nod is qualified)
    if price_response.price < nod_data.floor_price_minor {
        return Err(ContractError::NodNotQualified {
            current_price: price_response.price,
            floor_price: nod_data.floor_price_minor,
        });
    }

    // Create mint message for Gratis tokens using gratis_load_minor from Nod
    let mint_msg = Cw20ExecuteMsg::Mint {
        recipient: info.sender.to_string(),
        amount: nod_data.gratis_load_minor,
    };

    let mint_wasm_msg = WasmMsg::Execute {
        contract_addr: config.gratis_contract.to_string(),
        msg: to_json_binary(&mint_msg)?,
        funds: vec![],
    };

    // Create burn message for the Nod NFT
    let burn_msg = NodExecuteMsg::Burn {
        token_id: nod_token_id.clone(),
    };

    let burn_wasm_msg = WasmMsg::Execute {
        contract_addr: config.nod_contract.to_string(),
        msg: to_json_binary(&burn_msg)?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(mint_wasm_msg)
        .add_message(burn_wasm_msg)
        .add_attribute("method", "mine_gratis_with_nod")
        .add_attribute("miner", info.sender)
        .add_attribute("nod_token_id", nod_token_id)
        .add_attribute("amount", nod_data.gratis_load_minor)
        .add_attribute("current_price", price_response.price.atomics())
        .add_attribute("floor_price", nod_data.floor_price_minor.atomics())
        .add_attribute("gratis_contract", config.gratis_contract)
        .add_attribute("nod_contract", config.nod_contract))
}

/// Execute add to access list - admin only function
pub fn execute_add_to_access_list(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
    permissions: AccessPermissions,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    // Validate address
    let addr = deps.api.addr_validate(&address)?;

    // Check if address already exists
    if ACCESS_LIST.has(deps.storage, &addr) {
        return Err(ContractError::AddressAlreadyInAccessList {});
    }

    // Add to access list
    ACCESS_LIST.save(deps.storage, &addr, &permissions)?;

    Ok(Response::new()
        .add_attribute("method", "add_to_access_list")
        .add_attribute("admin", info.sender)
        .add_attribute("address", address)
        .add_attribute("can_mint_gratis", permissions.can_mint_gratis.to_string())
        .add_attribute("can_mint_promis", permissions.can_mint_promis.to_string()))
}

/// Execute remove from access list - admin only function
pub fn execute_remove_from_access_list(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    // Validate address
    let addr = deps.api.addr_validate(&address)?;

    // Cannot remove admin from access list
    if addr == config.admin {
        return Err(ContractError::CannotRemoveAdmin {});
    }

    // Check if address exists in access list
    if !ACCESS_LIST.has(deps.storage, &addr) {
        return Err(ContractError::AddressNotInAccessList {});
    }

    // Remove from access list
    ACCESS_LIST.remove(deps.storage, &addr);

    Ok(Response::new()
        .add_attribute("method", "remove_from_access_list")
        .add_attribute("admin", info.sender)
        .add_attribute("address", address))
}

/// Execute update permissions - admin only function
pub fn execute_update_permissions(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
    permissions: AccessPermissions,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    // Validate address
    let addr = deps.api.addr_validate(&address)?;

    // Check if address exists in access list
    if !ACCESS_LIST.has(deps.storage, &addr) {
        return Err(ContractError::AddressNotInAccessList {});
    }

    // Update permissions
    ACCESS_LIST.save(deps.storage, &addr, &permissions)?;

    Ok(Response::new()
        .add_attribute("method", "update_permissions")
        .add_attribute("admin", info.sender)
        .add_attribute("address", address)
        .add_attribute("can_mint_gratis", permissions.can_mint_gratis.to_string())
        .add_attribute("can_mint_promis", permissions.can_mint_promis.to_string()))
}

/// Execute transfer admin - admin only function
pub fn execute_transfer_admin(
    deps: DepsMut,
    info: MessageInfo,
    new_admin: String,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    // Validate new admin address
    let new_admin_addr = deps.api.addr_validate(&new_admin)?;

    // Cannot transfer to the same address
    if new_admin_addr == config.admin {
        return Err(ContractError::SameAdminAddress {});
    }

    // Update admin in config
    let old_admin = config.admin.clone();
    config.admin = new_admin_addr.clone();
    CONFIG.save(deps.storage, &config)?;

    // Add new admin to access list with full permissions if not already present
    if !ACCESS_LIST.has(deps.storage, &new_admin_addr) {
        let admin_permissions = AccessPermissions {
            can_mint_gratis: true,
            can_mint_promis: true,
            note: Some("Contract admin".to_string()),
        };
        ACCESS_LIST.save(deps.storage, &new_admin_addr, &admin_permissions)?;
    }

    Ok(Response::new()
        .add_attribute("method", "transfer_admin")
        .add_attribute("old_admin", old_admin)
        .add_attribute("new_admin", new_admin))
}

/// Execute update contracts - admin only function
pub fn execute_update_contracts(
    deps: DepsMut,
    info: MessageInfo,
    gratis_contract: Option<String>,
    promis_contract: Option<String>,
    price_oracle_contract: Option<String>,
    nod_contract: Option<String>,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    let mut response = Response::new()
        .add_attribute("method", "update_contracts")
        .add_attribute("admin", info.sender);

    // Update Gratis contract if provided
    if let Some(gratis_addr) = gratis_contract {
        let new_gratis_addr = deps.api.addr_validate(&gratis_addr)?;
        if new_gratis_addr == config.gratis_contract {
            return Err(ContractError::SameContractAddress {});
        }
        config.gratis_contract = new_gratis_addr;
        response = response.add_attribute("new_gratis_contract", gratis_addr);
    }

    // Update Promis contract if provided
    if let Some(promis_addr) = promis_contract {
        let new_promis_addr = deps.api.addr_validate(&promis_addr)?;
        if new_promis_addr == config.promis_contract {
            return Err(ContractError::SameContractAddress {});
        }
        config.promis_contract = new_promis_addr;
        response = response.add_attribute("new_promis_contract", promis_addr);
    }

    // Update Price Oracle contract if provided
    if let Some(price_oracle_addr) = price_oracle_contract {
        let new_price_oracle_addr = deps.api.addr_validate(&price_oracle_addr)?;
        if new_price_oracle_addr == config.price_oracle_contract {
            return Err(ContractError::SameContractAddress {});
        }
        config.price_oracle_contract = new_price_oracle_addr;
        response = response.add_attribute("new_price_oracle_contract", price_oracle_addr);
    }

    // Update Nod contract if provided
    if let Some(nod_addr) = nod_contract {
        let new_nod_addr = deps.api.addr_validate(&nod_addr)?;
        if new_nod_addr == config.nod_contract {
            return Err(ContractError::SameContractAddress {});
        }
        config.nod_contract = new_nod_addr;
        response = response.add_attribute("new_nod_contract", nod_addr);
    }

    // Save updated config
    CONFIG.save(deps.storage, &config)?;

    Ok(response)
}

/// Contract query entry point
#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    crate::query::query(deps, _env, msg)
}
