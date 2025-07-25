#[cfg(test)]
mod test_token_miner {
    use crate::contract::{execute, instantiate, query};
    use crate::msg::{
        AccessListResponse, AccessPermissionsResponse, CanMintResponse, ConfigResponse, ExecuteMsg,
        InstantiateMsg, QueryMsg,
    };
    use crate::state::{AccessPermissions, TokenType};
    use crate::ContractError;
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env, MockApi};
    use cosmwasm_std::{
        from_json, to_json_binary, ContractResult, Decimal, SystemError, SystemResult, Timestamp,
        Uint128, WasmMsg, WasmQuery,
    };
    use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;
    use nod::msg::ExecuteMsg as NodExecuteMsg;
    use nod::query::QueryMsg as NodQueryMsg;
    use nod::types::{NodData, State as NodState};
    use outbe_nft::msg::NftInfoResponse;
    use outbe_utils::denom::{Currency, Denom};
    use price_oracle::query::QueryMsg as PriceOracleQueryMsg;
    use price_oracle::types::{DayType, TokenPairPrice};
    use std::str::FromStr;

    const ADMIN: &str = "admin";
    const USER1: &str = "user1";
    const USER2: &str = "user2";
    const GRATIS_CONTRACT: &str = "gratis_contract";
    const PROMIS_CONTRACT: &str = "promis_contract";
    const PRICE_ORACLE_CONTRACT: &str = "price_oracle_contract";
    const NOD_CONTRACT: &str = "nod_contract";

    fn default_instantiate_msg(api: &MockApi) -> InstantiateMsg {
        InstantiateMsg {
            gratis_contract: api.addr_make(GRATIS_CONTRACT).into_string(),
            promis_contract: api.addr_make(PROMIS_CONTRACT).to_string(),
            price_oracle_contract: api.addr_make(PRICE_ORACLE_CONTRACT).to_string(),
            nod_contract: api.addr_make(NOD_CONTRACT).to_string(),
            access_list: Vec::new(),
        }
    }

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let msg = default_instantiate_msg(&deps.api);
        let admin_addr = &deps.api.addr_make(ADMIN);
        let gratis_addr = &deps.api.addr_make(GRATIS_CONTRACT);
        let promis_addr = &deps.api.addr_make(PROMIS_CONTRACT);
        let price_oracle_addr = &deps.api.addr_make(PRICE_ORACLE_CONTRACT);
        let nod_addr = &deps.api.addr_make(NOD_CONTRACT);

        let info = message_info(admin_addr, &[]);

        // Instantiate the contract
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Check response attributes
        assert_eq!(res.attributes.len(), 6);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "instantiate");

        // Query config to verify setup
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
        let config_response: ConfigResponse = from_json(&res).unwrap();
        assert_eq!(config_response.config.admin, admin_addr);
        assert_eq!(config_response.config.gratis_contract, gratis_addr);
        assert_eq!(config_response.config.promis_contract, promis_addr);
        assert_eq!(
            config_response.config.price_oracle_contract,
            price_oracle_addr
        );
        assert_eq!(config_response.config.nod_contract, nod_addr);

        // Check that admin was added to access list with full permissions
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::AccessPermissions {
                address: admin_addr.to_string(),
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
        let msg = default_instantiate_msg(&deps.api);
        let admin_addr = &deps.api.addr_make(ADMIN);
        let user1_addr = deps.api.addr_make(USER1);
        let gratis_contract_addr = deps.api.addr_make(GRATIS_CONTRACT);
        let info = message_info(admin_addr, &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Mint Gratis tokens
        let mint_msg = ExecuteMsg::Mine {
            recipient: user1_addr.to_string(),
            amount: Uint128::from(1000u128),
            token_type: TokenType::Gratis,
        };
        let info = message_info(admin_addr, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, mint_msg).unwrap();

        // Check that a WASM message was created
        assert_eq!(res.messages.len(), 1);

        if let cosmwasm_std::CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            msg,
            funds: _,
        }) = &res.messages[0].msg
        {
            assert_eq!(contract_addr, &gratis_contract_addr.to_string());
            let expected_msg = Cw20ExecuteMsg::Mint {
                recipient: user1_addr.to_string(),
                amount: Uint128::from(1000u128),
            };
            assert_eq!(msg, &cosmwasm_std::to_json_binary(&expected_msg).unwrap());
        } else {
            panic!("Expected WasmMsg::Execute");
        }

        // Check response attributes
        assert_eq!(res.attributes[0].value, "mint");
        assert_eq!(res.attributes[1].value, admin_addr.to_string());
        assert_eq!(res.attributes[2].value, user1_addr.to_string());
        assert_eq!(res.attributes[3].value, "1000");
        assert_eq!(res.attributes[4].value, "Gratis");
    }

    #[test]
    fn test_mint_promis_success() {
        let mut deps = mock_dependencies();
        let msg = default_instantiate_msg(&deps.api);
        let admin_addr = &deps.api.addr_make(ADMIN);
        let user1_addr = deps.api.addr_make(USER1);
        let promis_contract_addr = deps.api.addr_make(PROMIS_CONTRACT);
        let info = message_info(admin_addr, &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Mint Promis tokens
        let mint_msg = ExecuteMsg::Mine {
            recipient: user1_addr.to_string(),
            amount: Uint128::from(500u128),
            token_type: TokenType::Promis,
        };
        let info = message_info(admin_addr, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, mint_msg).unwrap();

        // Check that a WASM message was created
        assert_eq!(res.messages.len(), 1);

        if let cosmwasm_std::CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            msg,
            funds: _,
        }) = &res.messages[0].msg
        {
            assert_eq!(contract_addr, &promis_contract_addr.to_string());
            let expected_msg = Cw20ExecuteMsg::Mint {
                recipient: user1_addr.to_string(),
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
        let msg = default_instantiate_msg(&deps.api);
        let admin_addr = &deps.api.addr_make(ADMIN);
        let user1_addr = deps.api.addr_make(USER1);
        let info = message_info(admin_addr, &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Try to mint with unauthorized user
        let mint_msg = ExecuteMsg::Mine {
            recipient: user1_addr.to_string(),
            amount: Uint128::from(1000u128),
            token_type: TokenType::Gratis,
        };
        let info = message_info(&user1_addr, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, mint_msg).unwrap_err();

        match err {
            ContractError::AddressNotInAccessList {} => {}
            _ => panic!("Expected AddressNotInAccessList error"),
        }
    }

    #[test]
    fn test_mint_zero_amount() {
        let mut deps = mock_dependencies();
        let msg = default_instantiate_msg(&deps.api);
        let admin_addr = &deps.api.addr_make(ADMIN);
        let user1_addr = deps.api.addr_make(USER1);
        let info = message_info(admin_addr, &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Try to mint zero amount
        let mint_msg = ExecuteMsg::Mine {
            recipient: user1_addr.to_string(),
            amount: Uint128::zero(),
            token_type: TokenType::Gratis,
        };
        let info = message_info(admin_addr, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, mint_msg).unwrap_err();

        match err {
            ContractError::InvalidAmount {} => {}
            _ => panic!("Expected InvalidAmount error"),
        }
    }

    #[test]
    fn test_add_to_access_list() {
        let mut deps = mock_dependencies();
        let admin_addr = &deps.api.addr_make(ADMIN);
        let user1_addr = deps.api.addr_make(USER1);
        let msg = default_instantiate_msg(&deps.api);
        let info = message_info(admin_addr, &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Add user to access list
        let permissions = AccessPermissions {
            can_mint_gratis: true,
            can_mint_promis: false,
            note: Some("Test user".to_string()),
        };
        let add_msg = ExecuteMsg::AddToAccessList {
            address: user1_addr.to_string(),
            permissions: permissions.clone(),
        };
        let info = message_info(admin_addr, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, add_msg).unwrap();

        // Check response
        assert_eq!(res.attributes[0].value, "add_to_access_list");
        assert_eq!(res.attributes[2].value, user1_addr.to_string());

        // Query permissions to verify
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::AccessPermissions {
                address: user1_addr.to_string(),
            },
        )
        .unwrap();
        let permissions_response: AccessPermissionsResponse = from_json(&res).unwrap();
        assert!(permissions_response.permissions.is_some());
        let saved_permissions = permissions_response.permissions.unwrap();
        assert!(saved_permissions.can_mint_gratis);
        assert!(!saved_permissions.can_mint_promis);
    }

    #[test]
    fn test_add_to_access_list_unauthorized() {
        let mut deps = mock_dependencies();
        let msg = default_instantiate_msg(&deps.api);
        let admin_addr = &deps.api.addr_make(ADMIN);
        let user1_addr = deps.api.addr_make(USER1);
        let user2_addr = &deps.api.addr_make(USER2);
        let info = message_info(admin_addr, &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Try to add user with non-admin account
        let permissions = AccessPermissions {
            can_mint_gratis: true,
            can_mint_promis: false,
            note: None,
        };
        let add_msg = ExecuteMsg::AddToAccessList {
            address: user2_addr.to_string(),
            permissions,
        };
        let info = message_info(&user1_addr, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, add_msg).unwrap_err();

        match err {
            ContractError::Unauthorized {} => {}
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[test]
    fn test_remove_from_access_list() {
        let mut deps = mock_dependencies();
        let msg = default_instantiate_msg(&deps.api);
        let admin_addr = &deps.api.addr_make(ADMIN);
        let user1_addr = deps.api.addr_make(USER1);
        let info = message_info(admin_addr, &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Add user first
        let permissions = AccessPermissions {
            can_mint_gratis: true,
            can_mint_promis: true,
            note: None,
        };
        let add_msg = ExecuteMsg::AddToAccessList {
            address: user1_addr.to_string(),
            permissions,
        };
        let info = message_info(admin_addr, &[]);
        execute(deps.as_mut(), mock_env(), info, add_msg).unwrap();

        // Remove user from access list
        let remove_msg = ExecuteMsg::RemoveFromAccessList {
            address: user1_addr.to_string(),
        };
        let info = message_info(admin_addr, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, remove_msg).unwrap();

        // Check response
        assert_eq!(res.attributes[0].value, "remove_from_access_list");
        assert_eq!(res.attributes[2].value, user1_addr.to_string());

        // Query permissions to verify removal
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::AccessPermissions {
                address: user1_addr.to_string(),
            },
        )
        .unwrap();
        let permissions_response: AccessPermissionsResponse = from_json(&res).unwrap();
        assert!(permissions_response.permissions.is_none());
    }

    #[test]
    fn test_cannot_remove_admin() {
        let mut deps = mock_dependencies();
        let msg = default_instantiate_msg(&deps.api);
        let admin_addr = &deps.api.addr_make(ADMIN);
        let info = message_info(admin_addr, &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Try to remove admin from access list
        let remove_msg = ExecuteMsg::RemoveFromAccessList {
            address: admin_addr.to_string(),
        };
        let info = message_info(admin_addr, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, remove_msg).unwrap_err();

        match err {
            ContractError::CannotRemoveAdmin {} => {}
            _ => panic!("Expected CannotRemoveAdmin error"),
        }
    }

    #[test]
    fn test_update_permissions() {
        let mut deps = mock_dependencies();
        let msg = default_instantiate_msg(&deps.api);
        let admin_addr = &deps.api.addr_make(ADMIN);
        let user1_addr = deps.api.addr_make(USER1);
        let info = message_info(admin_addr, &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Add user first
        let permissions = AccessPermissions {
            can_mint_gratis: true,
            can_mint_promis: false,
            note: Some("Initial".to_string()),
        };
        let add_msg = ExecuteMsg::AddToAccessList {
            address: user1_addr.to_string(),
            permissions,
        };
        let info = message_info(admin_addr, &[]);
        execute(deps.as_mut(), mock_env(), info, add_msg).unwrap();

        // Update permissions
        let new_permissions = AccessPermissions {
            can_mint_gratis: false,
            can_mint_promis: true,
            note: Some("Updated".to_string()),
        };
        let update_msg = ExecuteMsg::UpdatePermissions {
            address: user1_addr.to_string(),
            permissions: new_permissions.clone(),
        };
        let info = message_info(admin_addr, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, update_msg).unwrap();

        // Check response
        assert_eq!(res.attributes[0].value, "update_permissions");

        // Query permissions to verify update
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::AccessPermissions {
                address: user1_addr.to_string(),
            },
        )
        .unwrap();
        let permissions_response: AccessPermissionsResponse = from_json(&res).unwrap();
        let updated_permissions = permissions_response.permissions.unwrap();
        assert!(!updated_permissions.can_mint_gratis);
        assert!(updated_permissions.can_mint_promis);
    }

    #[test]
    fn test_transfer_admin() {
        let mut deps = mock_dependencies();
        let msg = default_instantiate_msg(&deps.api);
        let admin_addr = &deps.api.addr_make(ADMIN);
        let user1_addr = deps.api.addr_make(USER1);
        let info = message_info(admin_addr, &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Transfer admin to new user
        let transfer_msg = ExecuteMsg::TransferAdmin {
            new_admin: user1_addr.to_string(),
        };
        let info = message_info(admin_addr, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, transfer_msg).unwrap();

        // Check response
        assert_eq!(res.attributes[0].value, "transfer_admin");
        assert_eq!(res.attributes[1].value, admin_addr.to_string());
        assert_eq!(res.attributes[2].value, user1_addr.to_string());

        // Query config to verify new admin
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
        let config_response: ConfigResponse = from_json(&res).unwrap();
        assert_eq!(config_response.config.admin, user1_addr);

        // Check that new admin has full permissions
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::AccessPermissions {
                address: user1_addr.to_string(),
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
        let msg = default_instantiate_msg(&deps.api);
        let admin_addr = &deps.api.addr_make(ADMIN);
        let user1_addr = deps.api.addr_make(USER1);
        let user2_addr = &deps.api.addr_make(USER2);
        let info = message_info(admin_addr, &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Add user with limited permissions
        let permissions = AccessPermissions {
            can_mint_gratis: true,
            can_mint_promis: false,
            note: None,
        };
        let add_msg = ExecuteMsg::AddToAccessList {
            address: user1_addr.to_string(),
            permissions,
        };
        let info = message_info(admin_addr, &[]);
        execute(deps.as_mut(), mock_env(), info, add_msg).unwrap();

        // Check if user can mint Gratis
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::CanMint {
                address: user1_addr.to_string(),
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
                address: user1_addr.to_string(),
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
                address: user2_addr.to_string(),
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
        let msg = default_instantiate_msg(&deps.api);
        let admin_addr = &deps.api.addr_make(ADMIN);
        let user1_addr = deps.api.addr_make(USER1);
        let user2_addr = &deps.api.addr_make(USER2);
        let info = message_info(admin_addr, &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Add users to access list
        let permissions1 = AccessPermissions {
            can_mint_gratis: true,
            can_mint_promis: false,
            note: Some("User 1".to_string()),
        };
        let add_msg1 = ExecuteMsg::AddToAccessList {
            address: user1_addr.to_string(),
            permissions: permissions1,
        };
        let info = message_info(admin_addr, &[]);
        execute(deps.as_mut(), mock_env(), info, add_msg1).unwrap();

        let permissions2 = AccessPermissions {
            can_mint_gratis: false,
            can_mint_promis: true,
            note: Some("User 2".to_string()),
        };
        let add_msg2 = ExecuteMsg::AddToAccessList {
            address: user2_addr.to_string(),
            permissions: permissions2,
        };
        let info = message_info(admin_addr, &[]);
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
        assert!(addresses.contains(&admin_addr.to_string()));
        assert!(addresses.contains(&user1_addr.to_string()));
        assert!(addresses.contains(&user2_addr.to_string()));
    }

    #[test]
    fn test_mint_with_limited_permissions() {
        let mut deps = mock_dependencies();
        let msg = default_instantiate_msg(&deps.api);
        let admin_addr = &deps.api.addr_make(ADMIN);
        let user1_addr = deps.api.addr_make(USER1);
        let user2_addr = &deps.api.addr_make(USER1);
        let info = message_info(admin_addr, &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Add user with limited permissions (only Gratis)
        let permissions = AccessPermissions {
            can_mint_gratis: true,
            can_mint_promis: false,
            note: None,
        };
        let add_msg = ExecuteMsg::AddToAccessList {
            address: user1_addr.to_string(),
            permissions,
        };
        let info = message_info(admin_addr, &[]);
        execute(deps.as_mut(), mock_env(), info, add_msg).unwrap();

        // User should be able to mint Gratis
        let mint_msg = ExecuteMsg::Mine {
            recipient: user2_addr.to_string(),
            amount: Uint128::from(1000u128),
            token_type: TokenType::Gratis,
        };
        let info = message_info(&user1_addr, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, mint_msg).unwrap();
        assert_eq!(res.messages.len(), 1);

        // User should NOT be able to mint Promis
        let mint_msg = ExecuteMsg::Mine {
            recipient: user2_addr.to_string(),
            amount: Uint128::from(1000u128),
            token_type: TokenType::Promis,
        };
        let info = message_info(&user1_addr, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, mint_msg).unwrap_err();

        match err {
            ContractError::NoMintPermission { token_type } => {
                assert_eq!(token_type, "Promis");
            }
            _ => panic!("Expected NoMintPermission error"),
        }
    }

    // Helper function to create a mock Nod NFT data
    fn mock_nod_data(
        owner: &str,
        state: NodState,
        floor_price_minor: Decimal,
        gratis_load_minor: Uint128,
    ) -> NodData {
        NodData {
            nod_id: "test_nod_1".to_string(),
            settlement_currency: Denom::Fiat(Currency::Usd),
            symbolic_rate: Decimal::one(),
            floor_rate: Uint128::new(100),
            nominal_price_minor: Decimal::from_str("1000").unwrap(),
            issuance_price_minor: Decimal::from_str("900").unwrap(),
            gratis_load_minor,
            floor_price_minor,
            state,
            owner: owner.to_string(),
            issued_at: Timestamp::from_seconds(1234567890),
            qualified_at: None,
        }
    }

    // Helper function to create a mock TokenPairPrice
    fn mock_token_pair_price(price: Decimal) -> TokenPairPrice {
        TokenPairPrice {
            token1: Denom::Fiat(Currency::Usd),
            token2: Denom::Native("token".to_string()),
            day_type: DayType::Green,
            price,
        }
    }

    #[test]
    fn test_mine_gratis_with_nod_success() {
        let mut deps = mock_dependencies();
        let user1_addr = deps.api.addr_make(USER1);
        let nod_contract_addr = deps.api.addr_make(NOD_CONTRACT);
        let price_oracle_addr = deps.api.addr_make(PRICE_ORACLE_CONTRACT);

        // Setup mock querier to return proper responses
        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { contract_addr, msg } => {
                if contract_addr == &nod_contract_addr.to_string() {
                    let query_msg: NodQueryMsg = from_json(msg).unwrap();
                    match query_msg {
                        NodQueryMsg::NftInfo { token_id: _ } => {
                            let nod_data = mock_nod_data(
                                user1_addr.to_string().as_ref(),
                                NodState::Issued,
                                Decimal::from_str("100").unwrap(),
                                Uint128::new(500),
                            );
                            let response = NftInfoResponse {
                                extension: nod_data,
                                owner: user1_addr.clone(),
                                token_id: "test_nod_1".to_string(),
                            };
                            SystemResult::Ok(ContractResult::Ok(to_json_binary(&response).unwrap()))
                        }
                        _ => SystemResult::Err(SystemError::UnsupportedRequest {
                            kind: "Only NftInfo supported in tests".to_string(),
                        }),
                    }
                } else if contract_addr == &price_oracle_addr.to_string() {
                    let query_msg: PriceOracleQueryMsg = from_json(msg).unwrap();
                    match query_msg {
                        PriceOracleQueryMsg::GetPrice {} => {
                            let price_response =
                                mock_token_pair_price(Decimal::from_atomics(150u128, 0).unwrap());
                            SystemResult::Ok(ContractResult::Ok(
                                to_json_binary(&price_response).unwrap(),
                            ))
                        }
                        _ => SystemResult::Err(SystemError::UnsupportedRequest {
                            kind: "Only GetPrice supported in tests".to_string(),
                        }),
                    }
                } else {
                    SystemResult::Err(SystemError::InvalidRequest {
                        error: "Unknown contract".to_string(),
                        request: msg.clone(),
                    })
                }
            }
            _ => SystemResult::Err(SystemError::UnsupportedRequest {
                kind: "Only WasmQuery::Smart supported".to_string(),
            }),
        });

        // Instantiate contract
        let msg = default_instantiate_msg(&deps.api);
        let admin_addr = &deps.api.addr_make(ADMIN);
        let user1_addr = deps.api.addr_make(USER1);
        let nod_contract_addr = deps.api.addr_make(NOD_CONTRACT);
        let gratis_contract_addr = deps.api.addr_make(GRATIS_CONTRACT);
        let info = message_info(admin_addr, &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Mine Gratis with Nod
        let mine_msg = ExecuteMsg::MineGratisWithNod {
            nod_token_id: "test_nod_1".to_string(),
        };
        let info = message_info(&user1_addr, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, mine_msg).unwrap();

        // Check that two WASM messages were created (mint and burn)
        assert_eq!(res.messages.len(), 2);

        // Check mint message
        if let cosmwasm_std::CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            msg,
            funds: _,
        }) = &res.messages[0].msg
        {
            assert_eq!(contract_addr, &gratis_contract_addr.to_string());
            let expected_mint_msg = Cw20ExecuteMsg::Mint {
                recipient: user1_addr.to_string(),
                amount: Uint128::new(500), // gratis_load_minor
            };
            assert_eq!(msg, &to_json_binary(&expected_mint_msg).unwrap());
        } else {
            panic!("Expected first message to be mint WasmMsg::Execute");
        }

        // Check burn message
        if let cosmwasm_std::CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            msg,
            funds: _,
        }) = &res.messages[1].msg
        {
            assert_eq!(contract_addr, &nod_contract_addr.to_string());
            let expected_burn_msg = NodExecuteMsg::Burn {
                token_id: "test_nod_1".to_string(),
            };
            assert_eq!(msg, &to_json_binary(&expected_burn_msg).unwrap());
        } else {
            panic!("Expected second message to be burn WasmMsg::Execute");
        }

        // Check response attributes
        assert_eq!(res.attributes[0].value, "mine_gratis_with_nod");
        assert_eq!(res.attributes[1].value, user1_addr.to_string());
        assert_eq!(res.attributes[2].value, "test_nod_1");
        assert_eq!(res.attributes[3].value, "500"); // gratis_load_minor
    }

    #[test]
    fn test_mine_gratis_with_nod_not_owner() {
        let mut deps = mock_dependencies();
        let user2_addr = deps.api.addr_make(USER2);
        let nod_contract_addr = deps.api.addr_make(NOD_CONTRACT);

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { contract_addr, msg } => {
                if contract_addr == &nod_contract_addr.to_string() {
                    let nod_data = mock_nod_data(
                        user2_addr.to_string().as_ref(),
                        NodState::Issued,
                        Decimal::from_str("100").unwrap(),
                        Uint128::new(500),
                    );
                    let response = NftInfoResponse {
                        extension: nod_data,
                        owner: user2_addr.clone(),
                        token_id: "test_nod_1".to_string(),
                    };
                    SystemResult::Ok(ContractResult::Ok(to_json_binary(&response).unwrap()))
                } else {
                    SystemResult::Err(SystemError::InvalidRequest {
                        error: "Unknown contract".to_string(),
                        request: msg.clone(),
                    })
                }
            }
            _ => SystemResult::Err(SystemError::UnsupportedRequest {
                kind: "Only WasmQuery::Smart supported".to_string(),
            }),
        });

        // Instantiate contract
        let msg = default_instantiate_msg(&deps.api);
        let admin_addr = &deps.api.addr_make(ADMIN);
        let user1_addr = deps.api.addr_make(USER1);
        let info = message_info(admin_addr, &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Try to mine Gratis with Nod owned by someone else
        let mine_msg = ExecuteMsg::MineGratisWithNod {
            nod_token_id: "test_nod_1".to_string(),
        };
        let info = message_info(&user1_addr, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, mine_msg).unwrap_err();

        match err {
            ContractError::NotNodOwner {} => {}
            _ => panic!("Expected NotNodOwner error"),
        }
    }

    #[test]
    fn test_mine_gratis_with_nod_not_qualified() {
        let mut deps = mock_dependencies();
        let user1_addr = deps.api.addr_make(USER1);
        let nod_contract_addr = deps.api.addr_make(NOD_CONTRACT);
        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { contract_addr, msg } => {
                if contract_addr == &nod_contract_addr.to_string() {
                    let nod_data = mock_nod_data(
                        user1_addr.as_str(),
                        NodState::Qualified,
                        Decimal::from_str("100").unwrap(),
                        Uint128::new(500),
                    );
                    let response = NftInfoResponse {
                        extension: nod_data,
                        owner: user1_addr.clone(),
                        token_id: "test_nod_1".to_string(),
                    };
                    SystemResult::Ok(ContractResult::Ok(to_json_binary(&response).unwrap()))
                } else {
                    SystemResult::Err(SystemError::InvalidRequest {
                        error: "Unknown contract".to_string(),
                        request: msg.clone(),
                    })
                }
            }
            _ => SystemResult::Err(SystemError::UnsupportedRequest {
                kind: "Only WasmQuery::Smart supported".to_string(),
            }),
        });

        // Instantiate contract
        let msg = default_instantiate_msg(&deps.api);
        let admin_addr = &deps.api.addr_make(ADMIN);
        let info = message_info(admin_addr, &[]);
        let user1_addr = deps.api.addr_make(USER1);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Try to mine Gratis with Nod that is not in Issued state
        let mine_msg = ExecuteMsg::MineGratisWithNod {
            nod_token_id: "test_nod_1".to_string(),
        };
        let info = message_info(&user1_addr, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, mine_msg).unwrap_err();

        match err {
            ContractError::NodNotIssued {} => {}
            _ => panic!("Expected NodNotIssued error"),
        }
    }

    #[test]
    fn test_mine_gratis_with_nod_price_not_qualified() {
        let mut deps = mock_dependencies();
        let user1_addr = deps.api.addr_make(USER1);
        let nod_contract_addr = deps.api.addr_make(NOD_CONTRACT);
        let price_oracle_addr = deps.api.addr_make(PRICE_ORACLE_CONTRACT);

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { contract_addr, msg } => {
                if contract_addr == &nod_contract_addr.to_string() {
                    let nod_data = mock_nod_data(
                        user1_addr.as_str(),
                        NodState::Issued,
                        Decimal::from_str("200").unwrap(),
                        Uint128::new(500),
                    );
                    let response = NftInfoResponse {
                        extension: nod_data,
                        owner: user1_addr.clone(),
                        token_id: "test_nod_1".to_string(),
                    };
                    SystemResult::Ok(ContractResult::Ok(to_json_binary(&response).unwrap()))
                } else if contract_addr == &price_oracle_addr.to_string() {
                    // Price too low - create a decimal that when converted to atomics is less than 200
                    let price_response = mock_token_pair_price(Decimal::from_str("150").unwrap());
                    SystemResult::Ok(ContractResult::Ok(to_json_binary(&price_response).unwrap()))
                } else {
                    SystemResult::Err(SystemError::InvalidRequest {
                        error: "Unknown contract".to_string(),
                        request: msg.clone(),
                    })
                }
            }
            _ => SystemResult::Err(SystemError::UnsupportedRequest {
                kind: "Only WasmQuery::Smart supported".to_string(),
            }),
        });

        // Instantiate contract
        let msg = default_instantiate_msg(&deps.api);
        let admin_addr = &deps.api.addr_make(ADMIN);
        let user1_addr = deps.api.addr_make(USER1);
        let info = message_info(admin_addr, &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Try to mine Gratis with Nod when price is too low
        let mine_msg = ExecuteMsg::MineGratisWithNod {
            nod_token_id: "test_nod_1".to_string(),
        };
        let info = message_info(&user1_addr, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, mine_msg).unwrap_err();

        match err {
            ContractError::NodNotQualified {
                current_price,
                floor_price,
            } => {
                assert_eq!(current_price, Decimal::from_str("150").unwrap());
                assert_eq!(floor_price, Decimal::from_str("200").unwrap());
            }
            _ => panic!("Expected NodNotQualified error"),
        }
    }
}
