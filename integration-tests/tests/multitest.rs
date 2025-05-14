use crate::setup::{setup_test_env, DeployedContract, NATIVE_DENOM};
use cosmwasm_std::{Addr, Decimal, HexBinary, Uint128};
use cw_multi_test::{App, ContractWrapper, Executor};
use std::str::FromStr;
use tribute::msg::ExecuteMsg::Mint;
use tribute::msg::{ConsumptionUnitCollectionExtension, ConsumptionUnitEntity, MintExtension};
use tribute::query::{QueryMsg, TributeInfoResponse};

mod setup;

#[test]
fn test_tribute() {
    let (mut app, config) = setup_test_env();

    println!("📦 Deploy Consumption Unit");
    let cu = deploy_tribute(&mut app, config.owner_addr.clone());

    println!("🧪 Perform tests");
    app.execute_contract(
        config.owner_addr.clone(),
        cu.address.clone(),
        &Mint {
            token_id: "1".to_string(),
            owner: config.user_addr.to_string(),
            extension: Box::new(MintExtension {
                entity: ConsumptionUnitEntity {
                    token_id: "1".to_string(),
                    owner: config.user_addr.to_string(),
                    minor_value_settlement: Uint128::from(100u32),
                    hashes: vec![HexBinary::from_hex("872be89dd82bcc6cf949d718f9274a624c927cfc91905f2bbb72fa44c9ea876d").unwrap()],
                },
                signature: HexBinary::from_hex("eea361aa7fff68cf0b07bc7b6d5907ba46a144ed1b5af6900bd0f96dc6e73e5f6e88eacffc84c3b3f84f2a0099503cd716883e251834176afc8b8e01b85d90bc").unwrap(),
                public_key: HexBinary::from_hex("02c21cb8a373fb63ee91d6133edcd18aefd7fa804adb2a0a55b1cb2f6f8aef068d").unwrap(),
            }),
        },
        &[],
    )
    .unwrap();

    let response: outbe_nft::msg::TokensResponse = app
        .wrap()
        .query_wasm_smart(
            cu.address.clone(),
            &QueryMsg::Tokens {
                owner: config.user_addr.to_string(),
                start_after: None,
                limit: None,
            },
        )
        .unwrap();

    assert_eq!(response.tokens.len(), 1);
    assert_eq!(response.tokens.first(), Some(&"1".to_string()));

    let response: TributeInfoResponse = app
        .wrap()
        .query_wasm_smart(
            cu.address.clone(),
            &QueryMsg::NftInfo {
                token_id: "1".to_string(),
            },
        )
        .unwrap();

    assert_eq!(
        response.extension.minor_value_settlement,
        Uint128::from(100u32)
    );
}

fn deploy_tribute(app: &mut App, owner: Addr) -> DeployedContract {
    use tribute::contract::{execute, instantiate};
    use tribute::msg::InstantiateMsg;
    use tribute::query::query;

    let code = ContractWrapper::new(execute, instantiate, query);
    let code_id = app.store_code(Box::new(code));

    let instantiate_msg = InstantiateMsg {
        name: "consumption unit".to_string(),
        symbol: "cu".to_string(),
        collection_info_extension: ConsumptionUnitCollectionExtension {
            settlement_token: cw20::Denom::Cw20(app.api().addr_make("usdc")),
            symbolic_rate: Decimal::from_str("0.08").unwrap(),
            native_token: cw20::Denom::Native(NATIVE_DENOM.to_string()),
            price_oracle: app.api().addr_make("ORACLE"), // todo replace after implementation
        },
        minter: None,
        creator: None,
    };
    let address = app
        .instantiate_contract(
            code_id,
            owner,
            &instantiate_msg,
            &[],
            "consumption unit".to_string(),
            None,
        )
        .unwrap();
    DeployedContract { address, code_id }
}
