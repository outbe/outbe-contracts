use crate::setup::{setup_test_env, DeployedContract, NATIVE_DENOM};
use consumption_unit::msg::ExecuteMsg::Mint;
use consumption_unit::msg::{
    ConsumptionUnitCollectionExtension, ConsumptionUnitEntity, MintExtension,
};
use consumption_unit::query::{ConsumptionUnitInfoResponse, QueryMsg};
use cosmwasm_std::{Addr, Uint128};
use cw_multi_test::{App, ContractWrapper, Executor};

mod setup;

#[test]
fn test_consumption_unit() {
    let (mut app, config) = setup_test_env();

    println!("📦 Deploy Consumption Unit");
    let cu = deploy_consumption_unit(&mut app, config.owner_addr.clone());

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
                    consumption_value: Uint128::from(100u32),
                    nominal_quantity: Uint128::from(100u32),
                    nominal_currency: "usd".to_string(),
                    hashes: vec!["872be89dd82bcc6cf949d718f9274a624c927cfc91905f2bbb72fa44c9ea876d".to_string()],
                },
                vector: 1,
                signature: "987327f5e1879d8a4739cad9ce0ef3743e5470a6fd2e6d96e67e87701dbcc81b30c6b978d2c40dd022cb9514416ff911ecbe26e9d9d7e726ce82bebf1d41a258"
                    .to_string(),
                public_key: "02c21cb8a373fb63ee91d6133edcd18aefd7fa804adb2a0a55b1cb2f6f8aef068d"
                    .to_string(),
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

    let response: ConsumptionUnitInfoResponse = app
        .wrap()
        .query_wasm_smart(
            cu.address.clone(),
            &QueryMsg::NftInfo {
                token_id: "1".to_string(),
            },
        )
        .unwrap();

    assert_eq!(response.extension.consumption_value, Uint128::from(100u32));
}

fn deploy_consumption_unit(app: &mut App, owner: Addr) -> DeployedContract {
    use consumption_unit::contract::{execute, instantiate};
    use consumption_unit::msg::InstantiateMsg;
    use consumption_unit::query::query;

    let code = ContractWrapper::new(execute, instantiate, query);
    let code_id = app.store_code(Box::new(code));

    let instantiate_msg = InstantiateMsg {
        name: "consumption unit".to_string(),
        symbol: "cu".to_string(),
        collection_info_extension: ConsumptionUnitCollectionExtension {
            settlement_token: cw20::Denom::Cw20(app.api().addr_make("usdc")),
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
