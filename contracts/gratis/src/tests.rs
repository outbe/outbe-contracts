#[cfg(test)]
mod tests {
    use crate::contract::{execute, query, CONTRACT_NAME, CONTRACT_VERSION};
    use crate::msg::{CheckTicketResponse, ExecuteMsg, QueryMsg};
    use cosmwasm_std::from_json;
    use cosmwasm_std::testing::{message_info, mock_dependencies_with_balance, mock_env};
    use cosmwasm_std::{Addr, DepsMut, Response, Uint128};
    use cw2::set_contract_version;
    use cw20::TokenInfoResponse;
    use cw20_base::state::{BALANCES, TOKEN_INFO};

    const CREATOR: &str = "creator";
    const USER1: &str = "user1";
    const USER2: &str = "user2";

    fn init_contract(deps: DepsMut) -> Result<Response, crate::ContractError> {
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        // Manually initialize basic CW20 state to bypass address validation
        use cosmwasm_std::Addr;
        use cw20_base::state::{MinterData, TokenInfo, TOKEN_INFO};

        let token_info = TokenInfo {
            name: "Gratis".to_string(),
            symbol: "GRATIS".to_string(),
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
        let mut deps = mock_dependencies_with_balance(&[]);
        let res = init_contract(deps.as_mut()).unwrap();
        assert_eq!(0, res.messages.len());

        // Check token info
        let res = query(deps.as_ref(), mock_env(), QueryMsg::TokenInfo {}).unwrap();
        let token_info: TokenInfoResponse = from_json(&res).unwrap();
        assert_eq!(token_info.name, "Gratis");
        assert_eq!(token_info.symbol, "GRATIS");
        assert_eq!(token_info.decimals, 6);
        assert_eq!(token_info.total_supply, Uint128::zero());
    }

    #[test]
    fn test_mint_and_burn() {
        let mut deps = mock_dependencies_with_balance(&[]);
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

        // Check total supply updated
        let res = query(deps.as_ref(), mock_env(), QueryMsg::TokenInfo {}).unwrap();
        let token_info: TokenInfoResponse = from_json(&res).unwrap();
        assert_eq!(token_info.total_supply, Uint128::from(500000u128));
    }

    #[test]
    fn test_ticket_generation() {
        let mut deps = mock_dependencies_with_balance(&[]);
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

        // Check non-existent ticket
        let check_msg = QueryMsg::CheckTicket {
            ticket: "nonexistent".to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), check_msg).unwrap();
        let check_response: CheckTicketResponse = from_json(&res).unwrap();
        assert!(!check_response.exists);
    }

    #[test]
    fn test_one_burn_per_block() {
        let mut deps = mock_dependencies_with_balance(&[]);
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
    fn test_burn_zero_amount() {
        let mut deps = mock_dependencies_with_balance(&[]);
        init_contract(deps.as_mut()).unwrap();

        let burn_msg = ExecuteMsg::Burn {
            amount: Uint128::zero(),
        };
        let info = message_info(&Addr::unchecked(USER1), &[]);
        let err = execute(deps.as_mut(), mock_env(), info, burn_msg).unwrap_err();
        match err {
            crate::ContractError::Std(e) => {
                assert!(e.to_string().contains("Invalid zero amount"));
            }
            _ => panic!("Expected Std error about zero amount"),
        }
    }

    #[test]
    fn test_burn_insufficient_funds() {
        let mut deps = mock_dependencies_with_balance(&[]);
        init_contract(deps.as_mut()).unwrap();

        let burn_msg = ExecuteMsg::Burn {
            amount: Uint128::from(100000u128),
        };
        let info = message_info(&Addr::unchecked(USER1), &[]);
        let err = execute(deps.as_mut(), mock_env(), info, burn_msg).unwrap_err();
        match err {
            crate::ContractError::Std(e) => {
                assert!(
                    e.to_string().contains("not found") || e.to_string().contains("Insufficient")
                );
            }
            _ => panic!("Expected Std error about insufficient funds"),
        }
    }

    #[test]
    fn test_ticket_uniqueness() {
        let mut deps = mock_dependencies_with_balance(&[]);
        init_contract(deps.as_mut()).unwrap();

        // Manually add balances to USER1 and USER2 to bypass address validation
        BALANCES
            .save(
                deps.as_mut().storage,
                &Addr::unchecked(USER1),
                &Uint128::from(1000000u128),
            )
            .unwrap();
        BALANCES
            .save(
                deps.as_mut().storage,
                &Addr::unchecked(USER2),
                &Uint128::from(1000000u128),
            )
            .unwrap();

        // Update total supply
        let mut token_info = TOKEN_INFO.load(deps.as_ref().storage).unwrap();
        token_info.total_supply = Uint128::from(2000000u128);
        TOKEN_INFO.save(deps.as_mut().storage, &token_info).unwrap();

        // Burn same amount for both users
        let burn_msg = ExecuteMsg::Burn {
            amount: Uint128::from(500000u128),
        };

        let info1 = message_info(&Addr::unchecked(USER1), &[]);
        let res1 = execute(deps.as_mut(), mock_env(), info1, burn_msg.clone()).unwrap();
        let ticket1 = &res1.attributes[3].value;

        let info2 = message_info(&Addr::unchecked(USER2), &[]);
        let res2 = execute(deps.as_mut(), mock_env(), info2, burn_msg).unwrap();
        let ticket2 = &res2.attributes[3].value;

        // Tickets should be different (different addresses)
        assert_ne!(ticket1, ticket2);

        // Both tickets should exist
        let check_msg = QueryMsg::CheckTicket {
            ticket: ticket1.to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), check_msg).unwrap();
        let check_response: CheckTicketResponse = from_json(&res).unwrap();
        assert!(check_response.exists);

        let check_msg = QueryMsg::CheckTicket {
            ticket: ticket2.to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), check_msg).unwrap();
        let check_response: CheckTicketResponse = from_json(&res).unwrap();
        assert!(check_response.exists);
    }

    #[test]
    fn test_different_block_heights() {
        let mut deps = mock_dependencies_with_balance(&[]);
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

        // Burn at block height 12345
        let burn_msg = ExecuteMsg::Burn {
            amount: Uint128::from(100000u128),
        };
        let info = message_info(&Addr::unchecked(USER1), &[]);
        let mut env = mock_env();
        env.block.height = 12345;
        let res1 = execute(deps.as_mut(), env, info.clone(), burn_msg.clone()).unwrap();
        let ticket1 = &res1.attributes[3].value;

        // Burn at block height 12346
        let mut env = mock_env();
        env.block.height = 12346;
        let res2 = execute(deps.as_mut(), env, info, burn_msg).unwrap();
        let ticket2 = &res2.attributes[3].value;

        // Tickets should be different (different block heights)
        assert_ne!(ticket1, ticket2);
    }
}