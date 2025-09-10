use cosmwasm_schema::cw_serde;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use cosmwasm_schema::schemars::JsonSchema;
use cosmwasm_schema::serde::{Deserialize, Serialize};
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    OverflowOperation, Response, StdResult, Uint128,
};

pub const CONTRACT_NAME: &str = "outbe.net:gratis";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
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
        ExecuteMsg::MintNative { recipient, amount } => {
            execute_mint_native(deps, env, info, recipient, amount)
        }
    }
}

pub fn execute_mint_native(
    _deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // Send native funds to sender
    let send_native_msg = create_mine_tokens_msg(
        env.contract.address.to_string(),
        recipient.to_string(),
        Coin {
            denom: "unit".to_string(),
            amount,
        },
    )?;

    let res = Response::new()
        .add_message(send_native_msg)
        .add_attribute("action", "mint_native")
        .add_attribute("amount", amount);

    Ok(res)
}

#[cw_serde]
struct MsgMineTokens {
    pub sender: String,
    pub recipient: String,
    pub amount: Coin,
}

fn create_mine_tokens_msg(
    sender: String,
    recipient: String,
    amount: Coin,
) -> Result<CosmosMsg, ContractError> {
    let serialized_msg = to_json_binary(&MsgMineTokens {
        sender,
        recipient,
        amount,
    })?;

    Ok(CosmosMsg::Stargate {
        type_url: "/outbe.tokenminer.MsgMineTokens".to_string(),
        value: serialized_msg,
    })
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    crate::query::query(deps, env, msg)
}
