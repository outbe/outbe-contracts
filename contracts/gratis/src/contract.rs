use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::native_mint::{MineTokensMsg, ProtoCoin};
use crate::state::{ADMIN, TICKETS, USER_BURNS_PER_BLOCK};
use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Env, MessageInfo, OverflowError, OverflowOperation,
    Response, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw20_base::contract::{execute as cw20_execute, instantiate as cw20_instantiate};
use cw20_base::msg::{ExecuteMsg as Cw20ExecuteMsg, InstantiateMsg as Cw20InstantiateMsg};
use cw20_base::state::{BALANCES, TOKEN_INFO};
use outbe_utils::gen_compound_hash;

pub const CONTRACT_NAME: &str = "outbe.net:gratis";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin_addr = msg
        .admin
        .map(|it| deps.api.addr_validate(&it))
        .unwrap_or(Ok(info.clone().sender))?;

    ADMIN.save(deps.storage, &admin_addr)?;

    let cw20_msg = Cw20InstantiateMsg {
        name: "Gratis".to_string(),
        symbol: "GRATIS".to_string(),
        decimals: 18,
        initial_balances: vec![],
        mint: msg.mint,
        marketing: None,
    };

    let res = cw20_instantiate(deps, env, info, cw20_msg)?;
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    match msg {
        MigrateMsg::Migrate {} => Ok(Response::new()),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Burn { amount } => execute_burn(deps, env, info, amount),
        ExecuteMsg::Mint { recipient, amount } => {
            let cw20_msg = Cw20ExecuteMsg::Mint { recipient, amount };
            Ok(cw20_execute(deps, env, info, cw20_msg)?)
        }
        ExecuteMsg::UpdateMinter { new_minter } => {
            execute_update_minter(deps, env, info, new_minter)
        }
        ExecuteMsg::UpdateAdmin { new_admin } => execute_update_admin(deps, env, info, new_admin),
        #[cfg(feature = "demo")]
        ExecuteMsg::MintNative { recipient, amount } => {
            execute_mint_native(deps, env, info, recipient, amount)
        }
    }
}

#[cfg(feature = "demo")]
pub fn execute_mint_native(
    _deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // Send native funds to sender
    let send_native_msg = MineTokensMsg {
        sender: env.contract.address.to_string(),
        recipient: recipient.clone(),
        amount: Some(ProtoCoin {
            denom: "unit".to_string(),
            amount: amount.to_string(),
        }),
    };

    let res = Response::new()
        .add_message(send_native_msg)
        .add_attribute("action", "mint_native")
        .add_attribute("recipient", recipient)
        .add_attribute("amount", amount);

    Ok(res)
}

pub fn execute_burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    if amount.is_zero() {
        #[allow(deprecated)]
        return Err(ContractError::Cw20Error(
            cw20_base::ContractError::InvalidZeroAmount {},
        ));
    }

    let block_height = env.block.height;
    let sender_address = info.sender.as_str();

    // Check if user already burned in this block
    let burn_key = (info.sender.clone(), block_height);
    if USER_BURNS_PER_BLOCK
        .may_load(deps.storage, burn_key.clone())?
        .is_some()
    {
        return Err(ContractError::AlreadyBurnedInBlock {});
    }
    let sender_balance = BALANCES.load(deps.storage, &info.sender)?;
    if sender_balance < amount {
        return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
            "Insufficient funds",
        )));
    }

    BALANCES.update(
        deps.storage,
        &info.sender,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        },
    )?;

    let mut token_info = TOKEN_INFO.load(deps.storage)?;
    token_info.total_supply = token_info.total_supply.checked_sub(amount).map_err(|_| {
        ContractError::Std(cosmwasm_std::StdError::overflow(OverflowError::new(
            OverflowOperation::Sub,
        )))
    })?;
    TOKEN_INFO.save(deps.storage, &token_info)?;

    // Mark that user has burned in this block
    USER_BURNS_PER_BLOCK.save(deps.storage, burn_key, &true)?;

    let ticket = gen_compound_hash(
        None,
        vec![
            sender_address.as_bytes(),
            amount.to_be_bytes().as_slice(),
            block_height.to_be_bytes().as_slice(),
        ],
    );

    TICKETS.save(deps.storage, ticket.to_hex(), &true)?;

    // Send native funds to sender
    let send_native_msg = MineTokensMsg {
        sender: env.contract.address.to_string(),
        recipient: info.sender.to_string(),
        amount: Some(ProtoCoin {
            denom: "unit".to_string(),
            amount: amount.to_string(),
        }),
    };

    let res = Response::new()
        .add_message(send_native_msg)
        .add_attribute("action", "burn")
        .add_attribute("from", sender_address)
        .add_attribute("amount", amount)
        .add_attribute("ticket", ticket.to_hex())
        .add_attribute("block_height", block_height.to_string());

    Ok(res)
}

pub fn execute_update_minter(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_minter: Option<String>,
) -> Result<Response, ContractError> {
    let admin = ADMIN.load(deps.storage)?;
    if info.sender != admin {
        return Err(ContractError::Unauthorized {});
    }

    // Update minter directly in token info
    let mut token_info = TOKEN_INFO.load(deps.storage)?;
    let old_minter = token_info.mint.as_ref().map(|m| m.minter.to_string());
    let new_minter_str = new_minter.clone();

    token_info.mint = match new_minter {
        Some(minter) => {
            let validated_minter = deps.api.addr_validate(&minter)?;
            Some(cw20_base::state::MinterData {
                minter: validated_minter,
                cap: token_info.mint.as_ref().and_then(|m| m.cap),
            })
        }
        None => None,
    };

    TOKEN_INFO.save(deps.storage, &token_info)?;

    let res = Response::new()
        .add_attribute("action", "update_minter")
        .add_attribute(
            "old_minter",
            old_minter.unwrap_or_else(|| "none".to_string()),
        )
        .add_attribute(
            "new_minter",
            new_minter_str.unwrap_or_else(|| "none".to_string()),
        );

    Ok(res)
}

pub fn execute_update_admin(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_admin: String,
) -> Result<Response, ContractError> {
    let admin = ADMIN.load(deps.storage)?;
    if info.sender != admin {
        return Err(ContractError::Unauthorized {});
    }

    let new_admin_addr = deps.api.addr_validate(&new_admin)?;
    ADMIN.save(deps.storage, &new_admin_addr)?;

    Ok(Response::new()
        .add_attribute("action", "update_admin")
        .add_attribute("old_admin", admin)
        .add_attribute("new_admin", new_admin_addr))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    crate::query::query(deps, env, msg)
}
