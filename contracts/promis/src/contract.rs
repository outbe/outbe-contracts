use blake3;
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, OverflowError,
    OverflowOperation, Response, StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20_base::contract::{
    execute as cw20_execute, instantiate as cw20_instantiate, query as cw20_query,
};
use cw20_base::msg::{
    ExecuteMsg as Cw20ExecuteMsg, InstantiateMsg as Cw20InstantiateMsg, QueryMsg as Cw20QueryMsg,
};
use cw20_base::state::{BALANCES, TOKEN_INFO};
use cw20_base::ContractError as Cw20ContractError;

use crate::error::ContractError;
use crate::msg::{CheckTicketResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{GRATIS_CONTRACT, TICKETS, USER_BURNS_PER_BLOCK};

const CONTRACT_NAME: &str = "outbe.net:promis";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Save Gratis contract address
    let gratis_addr = deps.api.addr_validate(&msg.gratis_contract)?;
    GRATIS_CONTRACT.save(deps.storage, &gratis_addr)?;

    let cw20_msg = Cw20InstantiateMsg {
        name: "Promis".to_string(),
        symbol: "PROMIS".to_string(),
        decimals: 6,
        initial_balances: vec![],
        mint: msg.mint,
        marketing: None,
    };

    let res = cw20_instantiate(deps, env, info, cw20_msg)?;
    Ok(res)
}

#[entry_point]
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
            let cw20_msg = Cw20ExecuteMsg::UpdateMinter { new_minter };
            Ok(cw20_execute(deps, env, info, cw20_msg)?)
        }
        ExecuteMsg::ConvertToGratis { amount } => {
            execute_convert_to_gratis(deps, env, info, amount)
        }
    }
}

pub fn execute_burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    if amount.is_zero() {
        return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
            "Invalid zero amount",
        )));
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

    let ticket_data = format!("{},{},{}", sender_address, amount, block_height);
    let hash = blake3::hash(ticket_data.as_bytes());
    let ticket = hash.to_hex().to_string();

    TICKETS.save(deps.storage, ticket.clone(), &true)?;

    let res = Response::new()
        .add_attribute("action", "burn")
        .add_attribute("from", sender_address)
        .add_attribute("amount", amount)
        .add_attribute("ticket", &ticket)
        .add_attribute("block_height", block_height.to_string());

    Ok(res)
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::CheckTicket { ticket } => to_json_binary(&query_check_ticket(deps, ticket)?),
        QueryMsg::Balance { address } => {
            let cw20_msg = Cw20QueryMsg::Balance { address };
            cw20_query(deps, env, cw20_msg)
        }
        QueryMsg::TokenInfo {} => {
            let cw20_msg = Cw20QueryMsg::TokenInfo {};
            cw20_query(deps, env, cw20_msg)
        }
        QueryMsg::Minter {} => {
            let cw20_msg = Cw20QueryMsg::Minter {};
            cw20_query(deps, env, cw20_msg)
        }
        QueryMsg::AllAccounts { start_after, limit } => {
            let cw20_msg = Cw20QueryMsg::AllAccounts { start_after, limit };
            cw20_query(deps, env, cw20_msg)
        }
    }
}

fn query_check_ticket(deps: Deps, ticket: String) -> StdResult<CheckTicketResponse> {
    let exists = TICKETS.may_load(deps.storage, ticket)?.unwrap_or(false);
    Ok(CheckTicketResponse { exists })
}

pub fn execute_convert_to_gratis(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    if amount.is_zero() {
        return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
            "Invalid zero amount",
        )));
    }

    // Check if user has enough Promis tokens
    let sender_balance = BALANCES.load(deps.storage, &info.sender)?;
    if sender_balance < amount {
        return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
            "Insufficient Promis balance",
        )));
    }

    // Burn the Promis tokens
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

    // Get Gratis contract address
    let gratis_contract = GRATIS_CONTRACT.load(deps.storage)?;

    // Create message to mint Gratis tokens 1:1
    let mint_msg = Cw20ExecuteMsg::Mint {
        recipient: info.sender.to_string(),
        amount,
    };

    let gratis_msg = WasmMsg::Execute {
        contract_addr: gratis_contract.to_string(),
        msg: to_json_binary(&mint_msg)?,
        funds: vec![],
    };

    let res = Response::new()
        .add_message(gratis_msg)
        .add_attribute("action", "convert_to_gratis")
        .add_attribute("from", &info.sender)
        .add_attribute("amount", amount)
        .add_attribute("gratis_contract", gratis_contract);

    Ok(res)
}

impl From<Cw20ContractError> for ContractError {
    fn from(err: Cw20ContractError) -> Self {
        match err {
            Cw20ContractError::Std(std_err) => ContractError::Std(std_err),
            Cw20ContractError::Unauthorized {} => ContractError::Unauthorized {},
            Cw20ContractError::CannotSetOwnAccount {} => ContractError::CannotSetOwnAccount {},
            Cw20ContractError::InvalidZeroAmount {} => {
                ContractError::Std(cosmwasm_std::StdError::generic_err("Invalid zero amount"))
            }
            Cw20ContractError::Expired {} => ContractError::Expired {},
            Cw20ContractError::NoAllowance {} => ContractError::NoAllowance {},
            Cw20ContractError::CannotExceedCap {} => ContractError::CannotExceedCap {},
            Cw20ContractError::LogoTooBig {} => ContractError::LogoTooBig {},
            Cw20ContractError::InvalidXmlPreamble {} => ContractError::InvalidXmlPreamble {},
            Cw20ContractError::InvalidPngHeader {} => ContractError::InvalidPngHeader {},
            Cw20ContractError::DuplicateInitialBalanceAddresses {} => {
                ContractError::DuplicateInitialBalanceAddresses {}
            }
            Cw20ContractError::InvalidExpiration {} => ContractError::Expired {},
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::{from_json, Addr, SubMsg, WasmMsg};
    use cw20::TokenInfoResponse;

    const CREATOR: &str = "creator";
    const GRATIS_CONTRACT: &str = "gratis_contract";
    const USER1: &str = "user1";

    fn init_contract(deps: DepsMut) -> Result<Response, ContractError> {
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        // Save contract addresses without validation for tests
        let gratis_addr = Addr::unchecked(GRATIS_CONTRACT);
        crate::state::GRATIS_CONTRACT.save(deps.storage, &gratis_addr)?;

        // Manually initialize basic CW20 state to bypass address validation
        use cosmwasm_std::Addr;
        use cw20_base::state::{MinterData, TokenInfo, TOKEN_INFO};

        let token_info = TokenInfo {
            name: "Promis".to_string(),
            symbol: "PROMIS".to_string(),
            decimals: 6,
            total_supply: Uint128::zero(),
            mint: Some(MinterData {
                minter: Addr::unchecked(CREATOR),
                cap: None,
            }),
        };

        TOKEN_INFO.save(deps.storage, &token_info)?;

        Ok(Response::new())
    }

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let res = init_contract(deps.as_mut()).unwrap();
        assert_eq!(0, res.messages.len());

        // Check token info
        let res = query(deps.as_ref(), mock_env(), QueryMsg::TokenInfo {}).unwrap();
        let token_info: TokenInfoResponse = from_json(&res).unwrap();
        assert_eq!(token_info.name, "Promis");
        assert_eq!(token_info.symbol, "PROMIS");
        assert_eq!(token_info.decimals, 6);
        assert_eq!(token_info.total_supply, Uint128::zero());
    }

    #[test]
    fn test_mint_and_burn() {
        let mut deps = mock_dependencies();
        init_contract(deps.as_mut()).unwrap();

        // Manually add balance to USER1 to bypass address validation
        use cw20_base::state::BALANCES;
        BALANCES
            .save(
                deps.as_mut().storage,
                &Addr::unchecked(USER1),
                &Uint128::from(1000000u128),
            )
            .unwrap();

        // Update total supply
        let mut token_info = TOKEN_INFO.load(deps.as_ref().storage).unwrap();
        token_info.total_supply = Uint128::from(1000000u128);
        TOKEN_INFO.save(deps.as_mut().storage, &token_info).unwrap();

        // Check balance directly from storage
        let balance = BALANCES
            .load(deps.as_ref().storage, &Addr::unchecked(USER1))
            .unwrap();
        assert_eq!(balance, Uint128::from(1000000u128));

        // Burn tokens
        let burn_msg = ExecuteMsg::Burn {
            amount: Uint128::from(500000u128),
        };
        let info = message_info(&Addr::unchecked(USER1), &[]);
        let res = execute(deps.as_mut(), mock_env(), info, burn_msg).unwrap();

        // Check burn response attributes
        assert_eq!(res.attributes.len(), 5);
        assert_eq!(res.attributes[0].key, "action");
        assert_eq!(res.attributes[0].value, "burn");
        assert_eq!(res.attributes[1].key, "from");
        assert_eq!(res.attributes[1].value, USER1);
        assert_eq!(res.attributes[2].key, "amount");
        assert_eq!(res.attributes[2].value, "500000");

        // Check updated balance directly from storage
        let balance = BALANCES
            .load(deps.as_ref().storage, &Addr::unchecked(USER1))
            .unwrap();
        assert_eq!(balance, Uint128::from(500000u128));
    }

    #[test]
    fn test_convert_to_gratis() {
        let mut deps = mock_dependencies();
        init_contract(deps.as_mut()).unwrap();

        // Manually add balance to USER1 to bypass address validation
        BALANCES
            .save(
                deps.as_mut().storage,
                &Addr::unchecked(USER1),
                &Uint128::from(1000000u128),
            )
            .unwrap();

        // Update total supply
        let mut token_info = TOKEN_INFO.load(deps.as_ref().storage).unwrap();
        token_info.total_supply = Uint128::from(1000000u128);
        TOKEN_INFO.save(deps.as_mut().storage, &token_info).unwrap();

        // Convert to Gratis
        let convert_msg = ExecuteMsg::ConvertToGratis {
            amount: Uint128::from(500000u128),
        };
        let info = message_info(&Addr::unchecked(USER1), &[]);
        let convert_res = execute(deps.as_mut(), mock_env(), info, convert_msg).unwrap();

        // Check conversion response
        assert_eq!(convert_res.messages.len(), 1);
        assert_eq!(convert_res.attributes.len(), 4);
        assert_eq!(convert_res.attributes[0].key, "action");
        assert_eq!(convert_res.attributes[0].value, "convert_to_gratis");
        assert_eq!(convert_res.attributes[1].key, "from");
        assert_eq!(convert_res.attributes[1].value, USER1);
        assert_eq!(convert_res.attributes[2].key, "amount");
        assert_eq!(convert_res.attributes[2].value, "500000");
        assert_eq!(convert_res.attributes[3].key, "gratis_contract");
        assert_eq!(convert_res.attributes[3].value, GRATIS_CONTRACT);

        // Check that Promis tokens were burned - directly from storage
        let balance = BALANCES
            .load(deps.as_ref().storage, &Addr::unchecked(USER1))
            .unwrap();
        assert_eq!(balance, Uint128::from(500000u128));

        // Check total supply updated
        let res = query(deps.as_ref(), mock_env(), QueryMsg::TokenInfo {}).unwrap();
        let token_info: TokenInfoResponse = from_json(&res).unwrap();
        assert_eq!(token_info.total_supply, Uint128::from(500000u128));

        // Check that a mint message was sent to Gratis contract
        let expected_mint_msg = Cw20ExecuteMsg::Mint {
            recipient: USER1.to_string(),
            amount: Uint128::from(500000u128),
        };
        let expected_wasm_msg = WasmMsg::Execute {
            contract_addr: GRATIS_CONTRACT.to_string(),
            msg: to_json_binary(&expected_mint_msg).unwrap(),
            funds: vec![],
        };
        assert_eq!(convert_res.messages[0], SubMsg::new(expected_wasm_msg));
    }

    #[test]
    fn test_convert_zero_amount() {
        let mut deps = mock_dependencies();
        init_contract(deps.as_mut()).unwrap();

        let convert_msg = ExecuteMsg::ConvertToGratis {
            amount: Uint128::zero(),
        };
        let info = message_info(&Addr::unchecked(USER1), &[]);
        let err = execute(deps.as_mut(), mock_env(), info, convert_msg).unwrap_err();
        match err {
            ContractError::Std(e) => {
                assert!(e.to_string().contains("Invalid zero amount"));
            }
            _ => panic!("Expected Std error about zero amount"),
        }
    }

    #[test]
    fn test_convert_insufficient_balance() {
        let mut deps = mock_dependencies();
        init_contract(deps.as_mut()).unwrap();

        let convert_msg = ExecuteMsg::ConvertToGratis {
            amount: Uint128::from(100000u128),
        };
        let info = message_info(&Addr::unchecked(USER1), &[]);
        let err = execute(deps.as_mut(), mock_env(), info, convert_msg).unwrap_err();
        match err {
            ContractError::Std(e) => {
                assert!(
                    e.to_string().contains("not found") || e.to_string().contains("Insufficient")
                );
            }
            _ => panic!("Expected Std error about insufficient balance"),
        }
    }

    #[test]
    fn test_one_burn_per_block() {
        let mut deps = mock_dependencies();
        init_contract(deps.as_mut()).unwrap();

        // Manually add balance to USER1 to bypass address validation
        BALANCES
            .save(
                deps.as_mut().storage,
                &Addr::unchecked(USER1),
                &Uint128::from(1000000u128),
            )
            .unwrap();

        // Update total supply
        let mut token_info = TOKEN_INFO.load(deps.as_ref().storage).unwrap();
        token_info.total_supply = Uint128::from(1000000u128);
        TOKEN_INFO.save(deps.as_mut().storage, &token_info).unwrap();

        // First burn should succeed
        let burn_msg = ExecuteMsg::Burn {
            amount: Uint128::from(100000u128),
        };
        let info = message_info(&Addr::unchecked(USER1), &[]);
        execute(deps.as_mut(), mock_env(), info.clone(), burn_msg.clone()).unwrap();

        // Second burn in same block should fail
        let err = execute(deps.as_mut(), mock_env(), info, burn_msg).unwrap_err();
        match err {
            ContractError::AlreadyBurnedInBlock {} => {}
            _ => panic!("Expected AlreadyBurnedInBlock error"),
        }
    }

    #[test]
    fn test_ticket_generation() {
        let mut deps = mock_dependencies();
        init_contract(deps.as_mut()).unwrap();

        // Manually add balance to USER1 to bypass address validation
        BALANCES
            .save(
                deps.as_mut().storage,
                &Addr::unchecked(USER1),
                &Uint128::from(1000000u128),
            )
            .unwrap();

        // Update total supply
        let mut token_info = TOKEN_INFO.load(deps.as_ref().storage).unwrap();
        token_info.total_supply = Uint128::from(1000000u128);
        TOKEN_INFO.save(deps.as_mut().storage, &token_info).unwrap();

        // Burn tokens
        let burn_msg = ExecuteMsg::Burn {
            amount: Uint128::from(500000u128),
        };
        let info = message_info(&Addr::unchecked(USER1), &[]);
        let res = execute(deps.as_mut(), mock_env(), info, burn_msg).unwrap();

        // Extract ticket from response
        let ticket = &res.attributes[3].value;

        // Check ticket exists
        let check_msg = QueryMsg::CheckTicket {
            ticket: ticket.to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), check_msg).unwrap();
        let check_response: CheckTicketResponse = from_json(&res).unwrap();
        assert!(check_response.exists);
    }

    #[test]
    fn test_multiple_conversions() {
        let mut deps = mock_dependencies();
        init_contract(deps.as_mut()).unwrap();

        // Manually add balance to USER1 to bypass address validation
        BALANCES
            .save(
                deps.as_mut().storage,
                &Addr::unchecked(USER1),
                &Uint128::from(1000000u128),
            )
            .unwrap();

        // Update total supply
        let mut token_info = TOKEN_INFO.load(deps.as_ref().storage).unwrap();
        token_info.total_supply = Uint128::from(1000000u128);
        TOKEN_INFO.save(deps.as_mut().storage, &token_info).unwrap();

        // First conversion
        let convert_msg = ExecuteMsg::ConvertToGratis {
            amount: Uint128::from(300000u128),
        };
        let info = message_info(&Addr::unchecked(USER1), &[]);
        execute(deps.as_mut(), mock_env(), info.clone(), convert_msg).unwrap();

        // Second conversion
        let convert_msg = ExecuteMsg::ConvertToGratis {
            amount: Uint128::from(200000u128),
        };
        execute(deps.as_mut(), mock_env(), info, convert_msg).unwrap();

        // Check remaining balance directly from storage
        let balance = BALANCES
            .load(deps.as_ref().storage, &Addr::unchecked(USER1))
            .unwrap();
        assert_eq!(balance, Uint128::from(500000u128));

        // Check total supply
        let res = query(deps.as_ref(), mock_env(), QueryMsg::TokenInfo {}).unwrap();
        let token_info: TokenInfoResponse = from_json(&res).unwrap();
        assert_eq!(token_info.total_supply, Uint128::from(500000u128));
    }
}
