use crate::setup::{setup_test_env, DeployedContract, NATIVE_DENOM};
use cosmwasm_std::{Addr, Decimal, HexBinary, Uint128};
use cw20::Denom;
use cw_multi_test::{App, ContractWrapper, Executor};
use std::str::FromStr;
use tribute::msg::ExecuteMsg::Mint;
use tribute::msg::{MintExtension, TributeCollectionExtension, TributeMintData};
use tribute::query::{QueryMsg, TributeInfoResponse};

mod setup;

#[test]
fn test_tribute() {
    let (mut app, config) = setup_test_env();

    println!("📦 Deploy Price Oracle");
    let price_oracle = deploy_price_oracle(&mut app, config.owner_addr.clone());
    println!("📦 Deploy Tribute");
    let tribute = deploy_tribute(&mut app, config.owner_addr.clone(), price_oracle.address);

    println!("🧪 Perform tests");
    app.execute_contract(
        config.owner_addr.clone(),
        tribute.address.clone(),
        &Mint {
            token_id: "1".to_string(),
            token_uri: None,
            owner: config.user_addr.to_string(),
            extension: Box::new(MintExtension {
                data: TributeMintData {
                    token_id: "1".to_string(),
                    owner: config.user_addr.to_string(),
                    settlement_token: Denom::Cw20(Addr::unchecked("usdc")),
                    settlement_value: Uint128::from(100000000u32),
                    hashes: vec![HexBinary::from_hex("872be89dd82bcc6cf949d718f9274a624c927cfc91905f2bbb72fa44c9ea876d").unwrap()],
                    tribute_date: None
                },
                signature: HexBinary::from_hex("3e0ecd67d3f2bd932abb5f7df70f6c5cc5923ba90e5b2992b0f9540970f4ffe7481d41858cef4f124036de071756d91c2369c40e5d6c1ce00812bfd08d102f42").unwrap(),
                public_key: HexBinary::from_hex("02c21cb8a373fb63ee91d6133edcd18aefd7fa804adb2a0a55b1cb2f6f8aef068d").unwrap(),
            }),
        },
        &[],
    )
    .unwrap();

    let response: outbe_nft::msg::TokensResponse = app
        .wrap()
        .query_wasm_smart(
            tribute.address.clone(),
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
            tribute.address.clone(),
            &QueryMsg::NftInfo {
                token_id: "1".to_string(),
            },
        )
        .unwrap();

    assert_eq!(
        response.extension.settlement_value,
        Uint128::from(100_000_000u64)
    );
}

#[test]
fn test_raffle() {
    let (mut app, config) = setup_test_env();

    println!("📦 Deploy Price Oracle");
    let price_oracle = deploy_price_oracle(&mut app, config.owner_addr.clone());
    println!("📦 Deploy Tribute");
    let tribute = deploy_tribute(
        &mut app,
        config.owner_addr.clone(),
        price_oracle.address.clone(),
    );

    println!("📦 Deploy Node");
    let nod = deploy_nod(&mut app, config.owner_addr.clone());

    println!("📦 Deploy Token Allocator");
    let token_allocator = deploy_token_allocator(&mut app, config.owner_addr.clone());

    println!("📦 Deploy Vector");
    let vector = deploy_vector(&mut app, config.owner_addr.clone());

    println!("📦 Deploy Raffle");
    let raffle = deploy_raffle(
        &mut app,
        config.owner_addr.clone(),
        tribute.address.clone(),
        nod.address.clone(),
        token_allocator.address.clone(),
        vector.address.clone(),
        price_oracle.address.clone(),
    );

    println!("🧪 Perform tests");

    println!("☑️ Add tribute");
    app.execute_contract(
        config.owner_addr.clone(),
        tribute.address.clone(),
        &Mint {
            token_id: "1".to_string(),
            token_uri: None,
            owner: config.user_addr.to_string(),
            extension: Box::new(MintExtension {
                data: TributeMintData {
                    token_id: "1".to_string(),
                    owner: config.user_addr.to_string(),
                    settlement_token: Denom::Cw20(Addr::unchecked("usdc")),
                    settlement_value: Uint128::from(5u32),
                    tribute_date: None,
                    hashes: vec![HexBinary::from_hex("872be89dd82bcc6cf949d718f9274a624c927cfc91905f2bbb72fa44c9ea876d").unwrap()],
                },
                signature: HexBinary::from_hex("504339506d74751f7660b5f57709d68929a5d394d33a9769dd65077aaf7070e67231067426f3647b5b67f35a8df05237ff7f4f37523bd5ca3289412d80b8af2c").unwrap(),
                public_key: HexBinary::from_hex("02c21cb8a373fb63ee91d6133edcd18aefd7fa804adb2a0a55b1cb2f6f8aef068d").unwrap(),
            }),
        },
        &[],
    )
        .unwrap();

    app.execute_contract(
        config.owner_addr.clone(),
        tribute.address.clone(),
        &Mint {
            token_id: "2".to_string(),
            token_uri: None,
            owner: config.user_addr.to_string(),
            extension: Box::new(MintExtension {
                data: TributeMintData {
                    token_id: "2".to_string(),
                    settlement_token: Denom::Cw20(Addr::unchecked("usdc")),
                    owner: config.user_addr.to_string(),
                    settlement_value: Uint128::from(15u32),
                    tribute_date: None,
                    hashes: vec![HexBinary::from_hex("02c21cb8a373fb63ee91d6133edcd18aefd7fa804adb2a0a55b1cb2f6f8aef068d").unwrap()],
                },
                signature: HexBinary::from_hex("b5188dd0dd759db3b3592708fb57665267d1268557a463b30d00f070239f69db290df316baaf6ff46c0afd9ab9f425b1f0ac05ab6fb333c78b2770748eed7b85").unwrap(),
                public_key: HexBinary::from_hex("02c21cb8a373fb63ee91d6133edcd18aefd7fa804adb2a0a55b1cb2f6f8aef068d").unwrap(),
            }),
        },
        &[],
    )
        .unwrap();

    let response: outbe_nft::msg::TokensResponse = app
        .wrap()
        .query_wasm_smart(
            tribute.address.clone(),
            &QueryMsg::AllTokens {
                start_after: None,
                limit: None,
            },
        )
        .unwrap();

    assert_eq!(response.tokens.len(), 2);

    let response: tribute::query::DailyTributesResponse = app
        .wrap()
        .query_wasm_smart(
            tribute.address.clone(),
            &QueryMsg::DailyTributes {
                date: app.block_info().time,
            },
        )
        .unwrap();

    assert_eq!(response.tributes.len(), 2);

    println!("🔬 Raffle 1");
    app.execute_contract(
        config.owner_addr.clone(),
        raffle.address.clone(),
        &lysis::msg::ExecuteMsg::Raffle { raffle_date: None },
        &[],
    )
    .unwrap();

    let response: outbe_nft::msg::TokensResponse = app
        .wrap()
        .query_wasm_smart(
            nod.address.clone(),
            &QueryMsg::AllTokens {
                start_after: None,
                limit: None,
            },
        )
        .unwrap();

    assert_eq!(response.tokens.len(), 2);

    println!("🔬 Raffle 2");
    app.execute_contract(
        config.owner_addr.clone(),
        raffle.address.clone(),
        &lysis::msg::ExecuteMsg::Raffle { raffle_date: None },
        &[],
    )
    .unwrap();

    let response: outbe_nft::msg::TokensResponse = app
        .wrap()
        .query_wasm_smart(
            nod.address.clone(),
            &QueryMsg::AllTokens {
                start_after: None,
                limit: None,
            },
        )
        .unwrap();

    assert_eq!(response.tokens.len(), 2);

    println!("🔬 Raffle 3");
    app.execute_contract(
        config.owner_addr.clone(),
        raffle.address.clone(),
        &lysis::msg::ExecuteMsg::Raffle { raffle_date: None },
        &[],
    )
    .unwrap();

    let response: outbe_nft::msg::TokensResponse = app
        .wrap()
        .query_wasm_smart(
            nod.address.clone(),
            &QueryMsg::AllTokens {
                start_after: None,
                limit: None,
            },
        )
        .unwrap();

    assert_eq!(
        response.tokens.len(),
        2,
        "No new nods because there were no tributes"
    );

    println!("🔬 Check distribution");
    let response: lysis::query::TributesDistributionResponse = app
        .wrap()
        .query_wasm_smart(
            raffle.address.clone(),
            &lysis::query::QueryMsg::TributesDistribution {},
        )
        .unwrap();

    assert_eq!(response.data.len(), 2,);
}

fn deploy_tribute(app: &mut App, owner: Addr, price_oracle: Addr) -> DeployedContract {
    use tribute::contract::{execute, instantiate};
    use tribute::msg::InstantiateMsg;
    use tribute::query::query;

    let code = ContractWrapper::new(execute, instantiate, query);
    let code_id = app.store_code(Box::new(code));

    let instantiate_msg = InstantiateMsg {
        name: "consumption unit".to_string(),
        symbol: "cu".to_string(),
        collection_info_extension: TributeCollectionExtension {
            symbolic_rate: Decimal::from_str("0.08").unwrap(),
            native_token: cw20::Denom::Native(NATIVE_DENOM.to_string()),
            price_oracle,
        },
        minter: None,
        creator: None,
        burner: None,
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
fn deploy_nod(app: &mut App, owner: Addr) -> DeployedContract {
    use nod::contract::{execute, instantiate};
    use nod::msg::InstantiateMsg;
    use nod::query::query;

    let code = ContractWrapper::new(execute, instantiate, query);
    let code_id = app.store_code(Box::new(code));

    let instantiate_msg = InstantiateMsg {
        name: "nod".to_string(),
        symbol: "nod".to_string(),
        collection_info_extension: nod::msg::NodCollectionExtension {},
        minter: None,
        creator: None,
        burner: None,
    };
    let address = app
        .instantiate_contract(
            code_id,
            owner,
            &instantiate_msg,
            &[],
            "nod".to_string(),
            None,
        )
        .unwrap();
    DeployedContract { address, code_id }
}

fn deploy_raffle(
    app: &mut App,
    owner: Addr,
    tribute: Addr,
    nod: Addr,
    token_allocator: Addr,
    vector: Addr,
    price_oracle: Addr,
) -> DeployedContract {
    use lysis::contract::{execute, instantiate};
    use lysis::msg::InstantiateMsg;
    use lysis::query::query;

    let code = ContractWrapper::new(execute, instantiate, query);
    let code_id = app.store_code(Box::new(code));

    let instantiate_msg = InstantiateMsg {
        creator: Some(owner.to_string()),
        vector: Some(vector),
        tribute: Some(tribute),
        nod: Some(nod),
        token_allocator: Some(token_allocator),
        price_oracle: Some(price_oracle),
        deficit: Decimal::from_str("0.08").unwrap(),
    };
    let address = app
        .instantiate_contract(
            code_id,
            owner,
            &instantiate_msg,
            &[],
            "lysis".to_string(),
            None,
        )
        .unwrap();
    DeployedContract { address, code_id }
}

fn deploy_price_oracle(app: &mut App, owner: Addr) -> DeployedContract {
    use price_oracle::contract::{execute, instantiate};
    use price_oracle::msg::InstantiateMsg;
    use price_oracle::query::query;

    let code = ContractWrapper::new(execute, instantiate, query);
    let code_id = app.store_code(Box::new(code));

    let instantiate_msg = InstantiateMsg {
        creator: None,
        initial_price: price_oracle::types::TokenPairPrice {
            token1: Denom::Native("one".to_string()),
            token2: Denom::Native("two".to_string()),
            day_type: price_oracle::types::DayType::GREEN,
            price: Decimal::from_str("1.25").unwrap(),
        },
    };
    let address = app
        .instantiate_contract(
            code_id,
            owner,
            &instantiate_msg,
            &[],
            "price-oracle".to_string(),
            None,
        )
        .unwrap();
    DeployedContract { address, code_id }
}

fn deploy_token_allocator(app: &mut App, owner: Addr) -> DeployedContract {
    use token_allocator::contract::{execute, instantiate};
    use token_allocator::msg::InstantiateMsg;
    use token_allocator::query::query;

    let code = ContractWrapper::new(execute, instantiate, query);
    let code_id = app.store_code(Box::new(code));

    let instantiate_msg = InstantiateMsg { creator: None };
    let address = app
        .instantiate_contract(
            code_id,
            owner,
            &instantiate_msg,
            &[],
            "token-allocator".to_string(),
            None,
        )
        .unwrap();
    DeployedContract { address, code_id }
}

fn deploy_vector(app: &mut App, owner: Addr) -> DeployedContract {
    use vector::contract::{execute, instantiate};
    use vector::msg::InstantiateMsg;
    use vector::query::query;

    let code = ContractWrapper::new(execute, instantiate, query);
    let code_id = app.store_code(Box::new(code));

    let instantiate_msg = InstantiateMsg {
        vectors: None,
        creator: None,
    };
    let address = app
        .instantiate_contract(
            code_id,
            owner,
            &instantiate_msg,
            &[],
            "vector".to_string(),
            None,
        )
        .unwrap();
    DeployedContract { address, code_id }
}
