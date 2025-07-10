use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{AccessPermissions, Config, TokenType, ACCESS_LIST, CONFIG};

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
    let gratis_contract = if cfg!(test) {
        Addr::unchecked(&msg.gratis_contract)
    } else {
        deps.api.addr_validate(&msg.gratis_contract)?
    };
    let promis_contract = if cfg!(test) {
        Addr::unchecked(&msg.promis_contract)
    } else {
        deps.api.addr_validate(&msg.promis_contract)?
    };

    // Create configuration with the instantiator as admin
    let config = Config {
        admin: info.sender.clone(),
        gratis_contract,
        promis_contract,
    };
    CONFIG.save(deps.storage, &config)?;

    // Add the admin to the access list with full permissions
    let admin_permissions = AccessPermissions {
        can_mint_gratis: true,
        can_mint_promis: true,
        note: Some("Contract admin".to_string()),
    };
    ACCESS_LIST.save(deps.storage, &info.sender, &admin_permissions)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", info.sender)
        .add_attribute("gratis_contract", msg.gratis_contract)
        .add_attribute("promis_contract", msg.promis_contract))
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
        ExecuteMsg::Mint {
            recipient,
            amount,
            token_type,
        } => execute_mint(deps, env, info, recipient, amount, token_type),
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
        } => execute_update_contracts(deps, info, gratis_contract, promis_contract),
    }
}

/// Execute mint function - mints tokens by calling the appropriate token contract
pub fn execute_mint(
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
    let _recipient_addr = if cfg!(test) {
        Addr::unchecked(&recipient)
    } else {
        deps.api.addr_validate(&recipient)?
    };

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
    let addr = if cfg!(test) {
        Addr::unchecked(&address)
    } else {
        deps.api.addr_validate(&address)?
    };

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
    let addr = if cfg!(test) {
        Addr::unchecked(&address)
    } else {
        deps.api.addr_validate(&address)?
    };

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
    let addr = if cfg!(test) {
        Addr::unchecked(&address)
    } else {
        deps.api.addr_validate(&address)?
    };

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
    let new_admin_addr = if cfg!(test) {
        Addr::unchecked(&new_admin)
    } else {
        deps.api.addr_validate(&new_admin)?
    };

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
        let new_gratis_addr = if cfg!(test) {
            Addr::unchecked(&gratis_addr)
        } else {
            deps.api.addr_validate(&gratis_addr)?
        };
        if new_gratis_addr == config.gratis_contract {
            return Err(ContractError::SameContractAddress {});
        }
        config.gratis_contract = new_gratis_addr;
        response = response.add_attribute("new_gratis_contract", gratis_addr);
    }

    // Update Promis contract if provided
    if let Some(promis_addr) = promis_contract {
        let new_promis_addr = if cfg!(test) {
            Addr::unchecked(&promis_addr)
        } else {
            deps.api.addr_validate(&promis_addr)?
        };
        if new_promis_addr == config.promis_contract {
            return Err(ContractError::SameContractAddress {});
        }
        config.promis_contract = new_promis_addr;
        response = response.add_attribute("new_promis_contract", promis_addr);
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