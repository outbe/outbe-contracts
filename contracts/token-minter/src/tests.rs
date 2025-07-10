#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate, query};
    use crate::msg::{
        AccessListResponse, AccessPermissionsResponse, CanMintResponse, ConfigResponse, ExecuteMsg,
        InstantiateMsg, QueryMsg,
    };
    use crate::state::{AccessPermissions, TokenType};
    use crate::ContractError;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, message_info};
    use cosmwasm_std::{from_json, Addr, Uint128, WasmMsg};
    use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;

    const ADMIN: &str = "admin";
    const USER1: &str = "user1";
    const USER2: &str = "user2";
    const GRATIS_CONTRACT: &str = "gratis_contract";
    const PROMIS_CONTRACT: &str = "promis_contract";

    fn default_instantiate_msg() -> InstantiateMsg {
        InstantiateMsg {
            gratis_contract: GRATIS_CONTRACT.to_string(),
            promis_contract: PROMIS_CONTRACT.to_string(),
        }
    }

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let msg = default_instantiate_msg();
        let info = message_info(&Addr::unchecked(ADMIN), &[]);

        // Instantiate the contract
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Check response attributes
        assert_eq!(res.attributes.len(), 4);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "instantiate");

        // Query config to verify setup
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
        let config_response: ConfigResponse = from_json(&res).unwrap();
        assert_eq!(config_response.config.admin, Addr::unchecked(ADMIN));
        assert_eq!(
            config_response.config.gratis_contract,
            Addr::unchecked(GRATIS_CONTRACT)
        );
        assert_eq!(
            config_response.config.promis_contract,
            Addr::unchecked(PROMIS_CONTRACT)
        );

        // Check that admin was added to access list with full permissions
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::AccessPermissions {
                address: ADMIN.to_string(),
            },
        )
        .unwrap();
        let permissions_response: AccessPermissionsResponse = from_json(&res).unwrap();
        assert!(permissions_response.permissions.is_some());
        let permissions = permissions_response.permissions.unwrap();
        assert!(permissions.can_mint_gratis);
        assert!(permissions.can_mint_promis);
    }

    #[test]
    fn test_mint_gratis_success() {
        let mut deps = mock_dependencies();
        let msg = default_instantiate_msg();
        let info = message_info(&Addr::unchecked(ADMIN), &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Mint Gratis tokens
        let mint_msg = ExecuteMsg::Mint {
            recipient: USER1.to_string(),
            amount: Uint128::from(1000u128),
            token_type: TokenType::Gratis,
        };
        let info = message_info(&Addr::unchecked(ADMIN), &[]);
        let res = execute(deps.as_mut(), mock_env(), info, mint_msg).unwrap();

        // Check that a WASM message was created
        assert_eq!(res.messages.len(), 1);

        if let cosmwasm_std::CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            msg,
            funds: _,
        }) = &res.messages[0].msg
        {
            assert_eq!(contract_addr, GRATIS_CONTRACT);
            let expected_msg = Cw20ExecuteMsg::Mint {
                recipient: USER1.to_string(),
                amount: Uint128::from(1000u128),
            };
            assert_eq!(msg, &cosmwasm_std::to_json_binary(&expected_msg).unwrap());
        } else {
            panic!("Expected WasmMsg::Execute");
        }

        // Check response attributes
        assert_eq!(res.attributes[0].value, "mint");
        assert_eq!(res.attributes[1].value, ADMIN);
        assert_eq!(res.attributes[2].value, USER1);
        assert_eq!(res.attributes[3].value, "1000");
        assert_eq!(res.attributes[4].value, "Gratis");
    }

    #[test]
    fn test_mint_promis_success() {
        let mut deps = mock_dependencies();
        let msg = default_instantiate_msg();
        let info = message_info(&Addr::unchecked(ADMIN), &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Mint Promis tokens
        let mint_msg = ExecuteMsg::Mint {
            recipient: USER1.to_string(),
            amount: Uint128::from(500u128),
            token_type: TokenType::Promis,
        };
        let info = message_info(&Addr::unchecked(ADMIN), &[]);
        let res = execute(deps.as_mut(), mock_env(), info, mint_msg).unwrap();

        // Check that a WASM message was created
        assert_eq!(res.messages.len(), 1);

        if let cosmwasm_std::CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            msg,
            funds: _,
        }) = &res.messages[0].msg
        {
            assert_eq!(contract_addr, PROMIS_CONTRACT);
            let expected_msg = Cw20ExecuteMsg::Mint {
                recipient: USER1.to_string(),
                amount: Uint128::from(500u128),
            };
            assert_eq!(msg, &cosmwasm_std::to_json_binary(&expected_msg).unwrap());
        } else {
            panic!("Expected WasmMsg::Execute");
        }
    }

    #[test]
    fn test_mint_unauthorized() {
        let mut deps = mock_dependencies();
        let msg = default_instantiate_msg();
        let info = message_info(&Addr::unchecked(ADMIN), &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Try to mint with unauthorized user
        let mint_msg = ExecuteMsg::Mint {
            recipient: USER1.to_string(),
            amount: Uint128::from(1000u128),
            token_type: TokenType::Gratis,
        };
        let info = message_info(&Addr::unchecked(USER1), &[]);
        let err = execute(deps.as_mut(), mock_env(), info, mint_msg).unwrap_err();

        match err {
            ContractError::AddressNotInAccessList {} => {}
            _ => panic!("Expected AddressNotInAccessList error"),
        }
    }

    #[test]
    fn test_mint_zero_amount() {
        let mut deps = mock_dependencies();
        let msg = default_instantiate_msg();
        let info = message_info(&Addr::unchecked(ADMIN), &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Try to mint zero amount
        let mint_msg = ExecuteMsg::Mint {
            recipient: USER1.to_string(),
            amount: Uint128::zero(),
            token_type: TokenType::Gratis,
        };
        let info = message_info(&Addr::unchecked(ADMIN), &[]);
        let err = execute(deps.as_mut(), mock_env(), info, mint_msg).unwrap_err();

        match err {
            ContractError::InvalidAmount {} => {}
            _ => panic!("Expected InvalidAmount error"),
        }
    }

    #[test]
    fn test_add_to_access_list() {
        let mut deps = mock_dependencies();
        let msg = default_instantiate_msg();
        let info = message_info(&Addr::unchecked(ADMIN), &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Add user to access list
        let permissions = AccessPermissions {
            can_mint_gratis: true,
            can_mint_promis: false,
            note: Some("Test user".to_string()),
        };
        let add_msg = ExecuteMsg::AddToAccessList {
            address: USER1.to_string(),
            permissions: permissions.clone(),
        };
        let info = message_info(&Addr::unchecked(ADMIN), &[]);
        let res = execute(deps.as_mut(), mock_env(), info, add_msg).unwrap();

        // Check response
        assert_eq!(res.attributes[0].value, "add_to_access_list");
        assert_eq!(res.attributes[2].value, USER1);

        // Query permissions to verify
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::AccessPermissions {
                address: USER1.to_string(),
            },
        )
        .unwrap();
        let permissions_response: AccessPermissionsResponse = from_json(&res).unwrap();
        assert!(permissions_response.permissions.is_some());
        let saved_permissions = permissions_response.permissions.unwrap();
        assert_eq!(saved_permissions.can_mint_gratis, true);
        assert_eq!(saved_permissions.can_mint_promis, false);
    }

    #[test]
    fn test_add_to_access_list_unauthorized() {
        let mut deps = mock_dependencies();
        let msg = default_instantiate_msg();
        let info = message_info(&Addr::unchecked(ADMIN), &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Try to add user with non-admin account
        let permissions = AccessPermissions {
            can_mint_gratis: true,
            can_mint_promis: false,
            note: None,
        };
        let add_msg = ExecuteMsg::AddToAccessList {
            address: USER2.to_string(),
            permissions,
        };
        let info = message_info(&Addr::unchecked(USER1), &[]);
        let err = execute(deps.as_mut(), mock_env(), info, add_msg).unwrap_err();

        match err {
            ContractError::Unauthorized {} => {}
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[test]
    fn test_remove_from_access_list() {
        let mut deps = mock_dependencies();
        let msg = default_instantiate_msg();
        let info = message_info(&Addr::unchecked(ADMIN), &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Add user first
        let permissions = AccessPermissions {
            can_mint_gratis: true,
            can_mint_promis: true,
            note: None,
        };
        let add_msg = ExecuteMsg::AddToAccessList {
            address: USER1.to_string(),
            permissions,
        };
        let info = message_info(&Addr::unchecked(ADMIN), &[]);
        execute(deps.as_mut(), mock_env(), info, add_msg).unwrap();

        // Remove user from access list
        let remove_msg = ExecuteMsg::RemoveFromAccessList {
            address: USER1.to_string(),
        };
        let info = message_info(&Addr::unchecked(ADMIN), &[]);
        let res = execute(deps.as_mut(), mock_env(), info, remove_msg).unwrap();

        // Check response
        assert_eq!(res.attributes[0].value, "remove_from_access_list");
        assert_eq!(res.attributes[2].value, USER1);

        // Query permissions to verify removal
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::AccessPermissions {
                address: USER1.to_string(),
            },
        )
        .unwrap();
        let permissions_response: AccessPermissionsResponse = from_json(&res).unwrap();
        assert!(permissions_response.permissions.is_none());
    }

    #[test]
    fn test_cannot_remove_admin() {
        let mut deps = mock_dependencies();
        let msg = default_instantiate_msg();
        let info = message_info(&Addr::unchecked(ADMIN), &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Try to remove admin from access list
        let remove_msg = ExecuteMsg::RemoveFromAccessList {
            address: ADMIN.to_string(),
        };
        let info = message_info(&Addr::unchecked(ADMIN), &[]);
        let err = execute(deps.as_mut(), mock_env(), info, remove_msg).unwrap_err();

        match err {
            ContractError::CannotRemoveAdmin {} => {}
            _ => panic!("Expected CannotRemoveAdmin error"),
        }
    }

    #[test]
    fn test_update_permissions() {
        let mut deps = mock_dependencies();
        let msg = default_instantiate_msg();
        let info = message_info(&Addr::unchecked(ADMIN), &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Add user first
        let permissions = AccessPermissions {
            can_mint_gratis: true,
            can_mint_promis: false,
            note: Some("Initial".to_string()),
        };
        let add_msg = ExecuteMsg::AddToAccessList {
            address: USER1.to_string(),
            permissions,
        };
        let info = message_info(&Addr::unchecked(ADMIN), &[]);
        execute(deps.as_mut(), mock_env(), info, add_msg).unwrap();

        // Update permissions
        let new_permissions = AccessPermissions {
            can_mint_gratis: false,
            can_mint_promis: true,
            note: Some("Updated".to_string()),
        };
        let update_msg = ExecuteMsg::UpdatePermissions {
            address: USER1.to_string(),
            permissions: new_permissions.clone(),
        };
        let info = message_info(&Addr::unchecked(ADMIN), &[]);
        let res = execute(deps.as_mut(), mock_env(), info, update_msg).unwrap();

        // Check response
        assert_eq!(res.attributes[0].value, "update_permissions");

        // Query permissions to verify update
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::AccessPermissions {
                address: USER1.to_string(),
            },
        )
        .unwrap();
        let permissions_response: AccessPermissionsResponse = from_json(&res).unwrap();
        let updated_permissions = permissions_response.permissions.unwrap();
        assert_eq!(updated_permissions.can_mint_gratis, false);
        assert_eq!(updated_permissions.can_mint_promis, true);
    }

    #[test]
    fn test_transfer_admin() {
        let mut deps = mock_dependencies();
        let msg = default_instantiate_msg();
        let info = message_info(&Addr::unchecked(ADMIN), &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Transfer admin to new user
        let transfer_msg = ExecuteMsg::TransferAdmin {
            new_admin: USER1.to_string(),
        };
        let info = message_info(&Addr::unchecked(ADMIN), &[]);
        let res = execute(deps.as_mut(), mock_env(), info, transfer_msg).unwrap();

        // Check response
        assert_eq!(res.attributes[0].value, "transfer_admin");
        assert_eq!(res.attributes[1].value, ADMIN);
        assert_eq!(res.attributes[2].value, USER1);

        // Query config to verify new admin
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
        let config_response: ConfigResponse = from_json(&res).unwrap();
        assert_eq!(config_response.config.admin, Addr::unchecked(USER1));

        // Check that new admin has full permissions
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::AccessPermissions {
                address: USER1.to_string(),
            },
        )
        .unwrap();
        let permissions_response: AccessPermissionsResponse = from_json(&res).unwrap();
        let permissions = permissions_response.permissions.unwrap();
        assert!(permissions.can_mint_gratis);
        assert!(permissions.can_mint_promis);
    }

    #[test]
    fn test_query_can_mint() {
        let mut deps = mock_dependencies();
        let msg = default_instantiate_msg();
        let info = message_info(&Addr::unchecked(ADMIN), &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Add user with limited permissions
        let permissions = AccessPermissions {
            can_mint_gratis: true,
            can_mint_promis: false,
            note: None,
        };
        let add_msg = ExecuteMsg::AddToAccessList {
            address: USER1.to_string(),
            permissions,
        };
        let info = message_info(&Addr::unchecked(ADMIN), &[]);
        execute(deps.as_mut(), mock_env(), info, add_msg).unwrap();

        // Check if user can mint Gratis
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::CanMint {
                address: USER1.to_string(),
                token_type: TokenType::Gratis,
            },
        )
        .unwrap();
        let can_mint_response: CanMintResponse = from_json(&res).unwrap();
        assert!(can_mint_response.can_mint);
        assert!(can_mint_response.reason.is_none());

        // Check if user can mint Promis
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::CanMint {
                address: USER1.to_string(),
                token_type: TokenType::Promis,
            },
        )
        .unwrap();
        let can_mint_response: CanMintResponse = from_json(&res).unwrap();
        assert!(!can_mint_response.can_mint);
        assert!(can_mint_response.reason.is_some());

        // Check unauthorized user
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::CanMint {
                address: USER2.to_string(),
                token_type: TokenType::Gratis,
            },
        )
        .unwrap();
        let can_mint_response: CanMintResponse = from_json(&res).unwrap();
        assert!(!can_mint_response.can_mint);
        assert!(can_mint_response.reason.is_some());
    }

    #[test]
    fn test_query_access_list() {
        let mut deps = mock_dependencies();
        let msg = default_instantiate_msg();
        let info = message_info(&Addr::unchecked(ADMIN), &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Add users to access list
        let permissions1 = AccessPermissions {
            can_mint_gratis: true,
            can_mint_promis: false,
            note: Some("User 1".to_string()),
        };
        let add_msg1 = ExecuteMsg::AddToAccessList {
            address: USER1.to_string(),
            permissions: permissions1,
        };
        let info = message_info(&Addr::unchecked(ADMIN), &[]);
        execute(deps.as_mut(), mock_env(), info, add_msg1).unwrap();

        let permissions2 = AccessPermissions {
            can_mint_gratis: false,
            can_mint_promis: true,
            note: Some("User 2".to_string()),
        };
        let add_msg2 = ExecuteMsg::AddToAccessList {
            address: USER2.to_string(),
            permissions: permissions2,
        };
        let info = message_info(&Addr::unchecked(ADMIN), &[]);
        execute(deps.as_mut(), mock_env(), info, add_msg2).unwrap();

        // Query access list
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::AccessList {
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
        let access_list_response: AccessListResponse = from_json(&res).unwrap();

        // Should include admin, user1, and user2
        assert_eq!(access_list_response.addresses.len(), 3);

        // Check that all addresses are present (order might vary)
        let addresses: Vec<String> = access_list_response
            .addresses
            .iter()
            .map(|(addr, _)| addr.to_string())
            .collect();
        assert!(addresses.contains(&ADMIN.to_string()));
        assert!(addresses.contains(&USER1.to_string()));
        assert!(addresses.contains(&USER2.to_string()));
    }

    #[test]
    fn test_mint_with_limited_permissions() {
        let mut deps = mock_dependencies();
        let msg = default_instantiate_msg();
        let info = message_info(&Addr::unchecked(ADMIN), &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Add user with limited permissions (only Gratis)
        let permissions = AccessPermissions {
            can_mint_gratis: true,
            can_mint_promis: false,
            note: None,
        };
        let add_msg = ExecuteMsg::AddToAccessList {
            address: USER1.to_string(),
            permissions,
        };
        let info = message_info(&Addr::unchecked(ADMIN), &[]);
        execute(deps.as_mut(), mock_env(), info, add_msg).unwrap();

        // User should be able to mint Gratis
        let mint_msg = ExecuteMsg::Mint {
            recipient: USER2.to_string(),
            amount: Uint128::from(1000u128),
            token_type: TokenType::Gratis,
        };
        let info = message_info(&Addr::unchecked(USER1), &[]);
        let res = execute(deps.as_mut(), mock_env(), info, mint_msg).unwrap();
        assert_eq!(res.messages.len(), 1);

        // User should NOT be able to mint Promis
        let mint_msg = ExecuteMsg::Mint {
            recipient: USER2.to_string(),
            amount: Uint128::from(1000u128),
            token_type: TokenType::Promis,
        };
        let info = message_info(&Addr::unchecked(USER1), &[]);
        let err = execute(deps.as_mut(), mock_env(), info, mint_msg).unwrap_err();

        match err {
            ContractError::NoMintPermission { token_type } => {
                assert_eq!(token_type, "Promis");
            }
            _ => panic!("Expected NoMintPermission error"),
        }
    }
}
