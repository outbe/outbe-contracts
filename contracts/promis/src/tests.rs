#[cfg(test)]
mod test_promis {
    use crate::contract::{execute, query, CONTRACT_NAME, CONTRACT_VERSION};
    use crate::msg::{CheckTicketResponse, ExecuteMsg, QueryMsg};
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::{
        from_json, to_json_binary, Addr, DepsMut, Response, SubMsg, Uint128, WasmMsg,
    };
    use cw2::set_contract_version;
    use cw20::{MinterResponse, TokenInfoResponse};
    use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;
    use cw20_base::state::{BALANCES, TOKEN_INFO};

    const CREATOR: &str = "creator";
    const TEST_ADMIN: &str = "admin";
    const GRATIS_CONTRACT: &str = "gratis_contract";
    const USER1: &str = "user1";
    const NEW_ADMIN: &str = "new_admin";
    const NEW_MINTER: &str = "new_minter";

    fn init_contract(deps: DepsMut) -> Result<Response, crate::ContractError> {
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        // Save contract addresses without validation for tests
        let gratis_addr = Addr::unchecked(GRATIS_CONTRACT);
        crate::state::GRATIS_CONTRACT.save(deps.storage, &gratis_addr)?;

        // Manually initialize basic CW20 state to bypass address validation
        use crate::state::ADMIN;
        use cosmwasm_std::Addr;
        use cw20_base::state::{MinterData, TokenInfo, TOKEN_INFO};

        let token_info = TokenInfo {
            name: "Promis".to_string(),
            symbol: "PROMIS".to_string(),
            decimals: 18,
            total_supply: Uint128::zero(),
            mint: Some(MinterData {
                minter: Addr::unchecked(CREATOR),
                cap: None,
            }),
        };

        TOKEN_INFO.save(deps.storage, &token_info)?;
        ADMIN.save(deps.storage, &Addr::unchecked(TEST_ADMIN))?;

        Ok(Response::new())
    }

    fn init_contract_with_admin(
        deps: DepsMut,
        admin: &str,
    ) -> Result<Response, crate::ContractError> {
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        // Save contract addresses without validation for tests
        let gratis_addr = Addr::unchecked(GRATIS_CONTRACT);
        crate::state::GRATIS_CONTRACT.save(deps.storage, &gratis_addr)?;

        // Manually initialize basic CW20 state to bypass address validation
        use crate::state::ADMIN;
        use cosmwasm_std::Addr;
        use cw20_base::state::{MinterData, TokenInfo, TOKEN_INFO};

        let token_info = TokenInfo {
            name: "Promis".to_string(),
            symbol: "PROMIS".to_string(),
            decimals: 18,
            total_supply: Uint128::zero(),
            mint: Some(MinterData {
                minter: Addr::unchecked(CREATOR),
                cap: None,
            }),
        };

        TOKEN_INFO.save(deps.storage, &token_info)?;
        ADMIN.save(deps.storage, &Addr::unchecked(admin))?;

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
        assert_eq!(token_info.decimals, 18);
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
            crate::ContractError::Std(e) => {
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
            crate::ContractError::Std(e) => {
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
            crate::ContractError::AlreadyBurnedInBlock {} => {}
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

    #[test]
    fn test_admin_instantiation() {
        let mut deps = mock_dependencies();
        let _res = init_contract_with_admin(deps.as_mut(), TEST_ADMIN).unwrap();

        // Check admin is set correctly
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Admin {}).unwrap();
        let admin_addr: String = from_json(&res).unwrap();
        assert_eq!(admin_addr, TEST_ADMIN);
    }

    #[test]
    fn test_admin_update_minter() {
        let mut deps = mock_dependencies();
        let admin_addr = &deps.api.addr_make(TEST_ADMIN);
        let new_minter_addr = &deps.api.addr_make(NEW_MINTER);
        let _res = init_contract_with_admin(deps.as_mut(), &admin_addr.to_string()).unwrap();

        // Admin should be able to update minter
        let update_msg = ExecuteMsg::UpdateMinter {
            new_minter: Some(new_minter_addr.to_string()),
        };
        let info = message_info(&admin_addr, &[]);
        let _res = execute(deps.as_mut(), mock_env(), info, update_msg).unwrap();

        // Check minter was updated
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Minter {}).unwrap();
        let minter_response: MinterResponse = from_json(&res).unwrap();
        assert_eq!(minter_response.minter, new_minter_addr.to_string());
    }

    #[test]
    fn test_non_admin_cannot_update_minter() {
        let mut deps = mock_dependencies();
        let _res = init_contract_with_admin(deps.as_mut(), TEST_ADMIN).unwrap();

        // Non-admin should not be able to update minter
        let update_msg = ExecuteMsg::UpdateMinter {
            new_minter: Some(NEW_MINTER.to_string()),
        };
        let info = message_info(&Addr::unchecked(USER1), &[]);
        let err = execute(deps.as_mut(), mock_env(), info, update_msg).unwrap_err();
        match err {
            crate::ContractError::Unauthorized {} => {}
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[test]
    fn test_admin_update_admin() {
        let mut deps = mock_dependencies();
        let admin_addr = &deps.api.addr_make(TEST_ADMIN);
        let new_admin_addr = &deps.api.addr_make(NEW_ADMIN);
        let _res = init_contract_with_admin(deps.as_mut(), &admin_addr.to_string()).unwrap();

        // Admin should be able to update admin
        let update_msg = ExecuteMsg::UpdateAdmin {
            new_admin: new_admin_addr.to_string(),
        };
        let info = message_info(&admin_addr, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, update_msg).unwrap();

        // Check response attributes
        assert_eq!(res.attributes.len(), 3);
        assert_eq!(res.attributes[0].key, "action");
        assert_eq!(res.attributes[0].value, "update_admin");
        assert_eq!(res.attributes[1].key, "old_admin");
        assert_eq!(res.attributes[1].value, admin_addr.to_string());
        assert_eq!(res.attributes[2].key, "new_admin");
        assert_eq!(res.attributes[2].value, new_admin_addr.to_string());

        // Check admin was updated
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Admin {}).unwrap();
        let admin_addr: String = from_json(&res).unwrap();
        assert_eq!(admin_addr, new_admin_addr.to_string());
    }

    #[test]
    fn test_non_admin_cannot_update_admin() {
        let mut deps = mock_dependencies();
        let admin_addr = &deps.api.addr_make(TEST_ADMIN);
        let new_admin_addr = &deps.api.addr_make(NEW_ADMIN);
        let user1_admin_addr = &deps.api.addr_make(USER1);
        let _res = init_contract_with_admin(deps.as_mut(), &admin_addr.to_string()).unwrap();

        // Non-admin should not be able to update admin
        let update_msg = ExecuteMsg::UpdateAdmin {
            new_admin: new_admin_addr.to_string(),
        };
        let info = message_info(&user1_admin_addr, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, update_msg).unwrap_err();
        match err {
            crate::ContractError::Unauthorized {} => {}
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[test]
    fn test_admin_transfer_workflow() {
        let mut deps = mock_dependencies();
        let admin_addr = &deps.api.addr_make(TEST_ADMIN);
        let new_admin_addr = &deps.api.addr_make(NEW_ADMIN);
        let new_minter_addr = &deps.api.addr_make(NEW_MINTER);
        let _res = init_contract_with_admin(deps.as_mut(), &admin_addr.to_string()).unwrap();

        // Original admin updates admin to new admin
        let update_msg = ExecuteMsg::UpdateAdmin {
            new_admin: new_admin_addr.to_string(),
        };
        let info = message_info(&admin_addr, &[]);
        let _res = execute(deps.as_mut(), mock_env(), info, update_msg).unwrap();

        // Original admin should no longer be able to update minter
        let update_msg = ExecuteMsg::UpdateMinter {
            new_minter: Some(new_minter_addr.to_string()),
        };
        let info = message_info(&admin_addr, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, update_msg).unwrap_err();
        match err {
            crate::ContractError::Unauthorized {} => {}
            _ => panic!("Expected Unauthorized error"),
        }

        // New admin should be able to update minter
        let update_msg = ExecuteMsg::UpdateMinter {
            new_minter: Some(new_minter_addr.to_string()),
        };
        let info = message_info(&new_admin_addr, &[]);
        let _res = execute(deps.as_mut(), mock_env(), info, update_msg).unwrap();

        // Check minter was updated
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Minter {}).unwrap();
        let minter_response: MinterResponse = from_json(&res).unwrap();
        assert_eq!(minter_response.minter, new_minter_addr.to_string());
    }

    #[test]
    fn test_query_admin() {
        let mut deps = mock_dependencies();
        let admin_addr = &deps.api.addr_make(TEST_ADMIN);
        let _res = init_contract_with_admin(deps.as_mut(), &admin_addr.to_string()).unwrap();

        // Query admin
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Admin {}).unwrap();
        let admin: String = from_json(&res).unwrap();
        assert_eq!(admin, admin_addr.to_string());
    }
}
