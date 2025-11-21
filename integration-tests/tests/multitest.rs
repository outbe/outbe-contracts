use crate::setup::{setup_test_env, DeployedContract, NATIVE_DENOM};
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_multi_test::{App, ContractWrapper, Executor};
use outbe_utils::date::normalize_to_date;
use outbe_utils::denom::{Currency, Denom};
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
                    tribute_id: "1".to_string(),
                    owner: config.user_addr.to_string(),
                    settlement_currency: Denom::Fiat(Currency::Usd),
                    nominal_amount_minor: Uint128::from(100000000u32),
                    settlement_amount_minor: Uint128::from(100000000u32),
                    worldwide_day: normalize_to_date(&app.block_info().time),
                    nominal_price: Decimal::one(),
                },
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
                query_order: None,
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
        response.extension.settlement_amount_minor,
        Uint128::from(100_000_000u64)
    );
}

#[test]
fn test_metadosis() {
    let (mut app, config) = setup_test_env();

    println!("📦 Deploy Price Oracle");
    let price_oracle = deploy_price_oracle(&mut app, config.owner_addr.clone());
    println!("📦 Deploy Tribute");
    let tribute = deploy_tribute(
        &mut app,
        config.owner_addr.clone(),
        price_oracle.address.clone(),
    );

    println!("📦 Deploy Nod");
    let nod = deploy_nod(&mut app, config.owner_addr.clone());

    println!("📦 Deploy Random Oracle");
    let random_oracle = deploy_random_oracle(&mut app, config.owner_addr.clone());

    println!("📦 Deploy Token Allocator");
    let token_allocator = deploy_token_allocator(&mut app, config.owner_addr.clone());

    println!("📦 Deploy Metadosis");
    let metadosis = deploy_metadosis(
        &mut app,
        config.owner_addr.clone(),
        tribute.address.clone(),
        nod.address.clone(),
        token_allocator.address.clone(),
        price_oracle.address.clone(),
        random_oracle.address.clone(),
    );
    println!("🧪 Perform tests");

    println!("☑️ Add token pair");

    app.execute_contract(
        config.owner_addr.clone(),
        price_oracle.address.clone(),
        &price_oracle::msg::ExecuteMsg::AddTokenPair {
            token1: Denom::Native("coen".to_string()),
            token2: Denom::Native("usdc".to_string()),
        },
        &[],
    )
    .unwrap();

    app.execute_contract(
        config.owner_addr.clone(),
        price_oracle.address.clone(),
        &price_oracle::msg::ExecuteMsg::AddTokenPair {
            token1: Denom::Native("xau".to_string()),
            token2: Denom::Fiat(Currency::Usd),
        },
        &[],
    )
    .unwrap();

    println!("☑️ Set Green day");

    app.execute_contract(
        config.owner_addr.clone(),
        price_oracle.address.clone(),
        &price_oracle::msg::ExecuteMsg::SetDayType {
            token1: Denom::Native("coen".to_string()),
            token2: Denom::Native("usdc".to_string()),
            day_type: price_oracle::types::DayType::Green,
        },
        &[],
    )
    .unwrap();

    app.execute_contract(
        config.owner_addr.clone(),
        price_oracle.address.clone(),
        &price_oracle::msg::ExecuteMsg::SetDayType {
            token1: Denom::Native("xau".to_string()),
            token2: Denom::Fiat(Currency::Usd),
            day_type: price_oracle::types::DayType::Green,
        },
        &[],
    )
    .unwrap();

    println!("☑️ Add price");
    app.execute_contract(
        config.owner_addr.clone(),
        price_oracle.address.clone(),
        &price_oracle::msg::ExecuteMsg::UpdatePrice {
            token1: Denom::Native("coen".to_string()),
            token2: Denom::Native("usdc".to_string()),
            price: Decimal::from_str("1.25").unwrap(),
            open: None,
            close: None,
            high: None,
            low: None,
            volume: None,
        },
        &[],
    )
    .unwrap();

    app.execute_contract(
        config.owner_addr.clone(),
        price_oracle.address.clone(),
        &price_oracle::msg::ExecuteMsg::UpdatePrice {
            token1: Denom::Native("xau".to_string()),
            token2: Denom::Fiat(Currency::Usd),
            price: Decimal::from_str("3305.90").unwrap(),
            open: None,
            close: None,
            high: None,
            low: None,
            volume: None,
        },
        &[],
    )
    .unwrap();

    println!("☑️ Add tributes");
    app.execute_contract(
        config.owner_addr.clone(),
        tribute.address.clone(),
        &Mint {
            token_id: "1".to_string(),
            token_uri: None,
            owner: config.user_addr.to_string(),
            extension: Box::new(MintExtension {
                data: TributeMintData {
                    tribute_id: "1".to_string(),
                    owner: config.user_addr.to_string(),
                    settlement_currency: Denom::Fiat(Currency::Usd),
                    settlement_amount_minor: Uint128::from(5_000000000000000000u128),
                    nominal_amount_minor: Uint128::from(10_000000000000000000u128),
                    worldwide_day: normalize_to_date(&app.block_info().time),
                    nominal_price: Decimal::from_str("0.5").unwrap(),
                },
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
                    tribute_id: "2".to_string(),
                    settlement_currency: Denom::Fiat(Currency::Usd),
                    owner: config.user_addr.to_string(),
                    settlement_amount_minor: Uint128::from(150_000000000000000000u128),
                    nominal_amount_minor: Uint128::from(5_000000000000000000u128),
                    worldwide_day: normalize_to_date(&app.block_info().time),
                    nominal_price: Decimal::from_str("3").unwrap(),
                },
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
                query_order: None,
            },
        )
        .unwrap();

    assert_eq!(response.tokens.len(), 2);

    let response: tribute::query::FullTributesResponse = app
        .wrap()
        .query_wasm_smart(
            tribute.address.clone(),
            &QueryMsg::DailyTributes {
                date: Some(normalize_to_date(&app.block_info().time)),
                start_after: None,
                limit: None,
                query_order: None,
            },
        )
        .unwrap();

    assert_eq!(response.tributes.len(), 2);

    println!("🔬 Metadosis: Prepare");
    app.execute_contract(
        config.owner_addr.clone(),
        metadosis.address.clone(),
        &metadosis::msg::ExecuteMsg::Prepare { run_date: None },
        &[],
    )
    .unwrap();

    let response: metadosis::query::MetadosisInfoResponse = app
        .wrap()
        .query_wasm_smart(
            metadosis.address.clone(),
            &metadosis::query::QueryMsg::MetadosisInfo {},
        )
        .unwrap();

    assert_eq!(response.data.len(), 1);
    let metadosis_info = response.data.first().unwrap();
    println!("Metadosis info: {:?}", metadosis_info);

    println!("🔬 Lysis 1");
    app.execute_contract(
        config.owner_addr.clone(),
        metadosis.address.clone(),
        &metadosis::msg::ExecuteMsg::Execute { run_date: None },
        &[],
    )
    .unwrap();

    let _response: outbe_nft::msg::TokensResponse = app
        .wrap()
        .query_wasm_smart(
            nod.address.clone(),
            &QueryMsg::AllTokens {
                start_after: None,
                limit: None,
                query_order: None,
            },
        )
        .unwrap();
}

pub fn deploy_tribute(app: &mut App, owner: Addr, price_oracle: Addr) -> DeployedContract {
    use tribute::contract::{execute, instantiate};
    use tribute::msg::InstantiateMsg;
    use tribute::query::query;

    let code = ContractWrapper::new(execute, instantiate, query);
    let code_id = app.store_code(Box::new(code));

    let instantiate_msg = InstantiateMsg {
        name: "consumption unit".to_string(),
        symbol: "cu".to_string(),
        collection_info_extension: TributeCollectionExtension {
            native_token: Denom::Native(NATIVE_DENOM.to_string()),
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
pub fn deploy_nod(app: &mut App, owner: Addr) -> DeployedContract {
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

pub fn deploy_random_oracle(app: &mut App, owner: Addr) -> DeployedContract {
    use random_oracle::contract::{execute, instantiate, query};
    use random_oracle::msg::InstantiateMsg;

    let code = ContractWrapper::new(execute, instantiate, query);
    let code_id = app.store_code(Box::new(code));

    let instantiate_msg = InstantiateMsg {
        random_value: Some(123),
    };
    let address = app
        .instantiate_contract(
            code_id,
            owner,
            &instantiate_msg,
            &[],
            "randao".to_string(),
            None,
        )
        .unwrap();
    DeployedContract { address, code_id }
}

#[allow(clippy::too_many_arguments)]
pub fn deploy_metadosis(
    app: &mut App,
    owner: Addr,
    tribute: Addr,
    nod: Addr,
    token_allocator: Addr,
    price_oracle: Addr,
    random_oracle: Addr,
) -> DeployedContract {
    use metadosis::contract::{execute, instantiate, reply};
    use metadosis::msg::InstantiateMsg;
    use metadosis::query::query;

    let code = ContractWrapper::new(execute, instantiate, query).with_reply(reply);
    let code_id = app.store_code(Box::new(code));

    let instantiate_msg = InstantiateMsg {
        creator: Some(owner.to_string()),
        tribute: Some(tribute),
        nod: Some(nod),
        token_allocator: Some(token_allocator),
        price_oracle: Some(price_oracle),
        random_oracle: Some(random_oracle),
        lysis_limit_percent: Decimal::from_str("0.08").unwrap(),
    };
    let address = app
        .instantiate_contract(
            code_id,
            owner,
            &instantiate_msg,
            &[],
            "metadosis".to_string(),
            None,
        )
        .unwrap();
    DeployedContract { address, code_id }
}

pub fn deploy_price_oracle(app: &mut App, owner: Addr) -> DeployedContract {
    use price_oracle::contract::{execute, instantiate};
    use price_oracle::msg::InstantiateMsg;
    use price_oracle::query::query;

    let code = ContractWrapper::new(execute, instantiate, query);
    let code_id = app.store_code(Box::new(code));

    let instantiate_msg = InstantiateMsg {
        creator: None,
        vwap_window_seconds: Some(300),
        nod_address: None,
    };

    let address = app
        .instantiate_contract(
            code_id,
            owner.clone(),
            &instantiate_msg,
            &[],
            "price-oracle".to_string(),
            None,
        )
        .unwrap();

    DeployedContract { address, code_id }
}

pub fn deploy_token_allocator(app: &mut App, owner: Addr) -> DeployedContract {
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

#[allow(dead_code)]
pub fn deploy_vector(app: &mut App, owner: Addr) -> DeployedContract {
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

#[test]
fn test_price_oracle_nod_qualification() {
    use nod::msg::ExecuteMsg as NodExecuteMsg;
    use nod::msg::{NodEntity, SubmitExtension};
    use nod::query::QueryMsg as NodQueryMsg;
    use nod::types::State;
    use price_oracle::msg::ExecuteMsg as PriceOracleExecuteMsg;

    println!("🎯 Test Price Oracle NOD Qualification");
    let (mut app, config) = setup_test_env();

    let owner_addr = config.owner_addr;

    // Deploy contracts
    let price_oracle = deploy_price_oracle(&mut app, owner_addr.clone());
    let nod = deploy_nod(&mut app, owner_addr.clone());

    // Setup price oracle with nod address
    println!("🔗 Configure price oracle with nod address");
    let msg = PriceOracleExecuteMsg::UpdateNodAddress {
        nod_address: Some(nod.address.to_string()),
    };
    app.execute_contract(owner_addr.clone(), price_oracle.address.clone(), &msg, &[])
        .unwrap();

    // Add token pair to price oracle
    println!("🔥 Add coen/usdc token pair to price oracle");
    let coen_denom = Denom::Native("coen".to_string());
    let usdc_denom = Denom::Native("usdc".to_string());
    let msg = PriceOracleExecuteMsg::AddTokenPair {
        token1: coen_denom.clone(),
        token2: usdc_denom.clone(),
    };
    app.execute_contract(owner_addr.clone(), price_oracle.address.clone(), &msg, &[])
        .unwrap();

    // Setup price updater for nod contract BEFORE adding price
    println!("🔧 Set price oracle as price updater for nod contract");
    let msg = NodExecuteMsg::UpdatePriceUpdater {
        price_updater: Some(price_oracle.address.to_string()),
    };
    app.execute_contract(owner_addr.clone(), nod.address.clone(), &msg, &[])
        .unwrap();

    // Add initial price to oracle
    println!("💰 Add initial price to oracle: 0.3");
    let msg = PriceOracleExecuteMsg::UpdatePrice {
        token1: coen_denom.clone(),
        token2: usdc_denom.clone(),
        price: Decimal::from_str("0.3").unwrap(),
        open: Some(Decimal::from_str("0.29").unwrap()),
        high: Some(Decimal::from_str("0.32").unwrap()),
        low: Some(Decimal::from_str("0.28").unwrap()),
        close: Some(Decimal::from_str("0.3").unwrap()),
        volume: Some(Uint128::new(1000)),
    };
    app.execute_contract(owner_addr.clone(), price_oracle.address.clone(), &msg, &[])
        .unwrap();

    // Create nod tokens with different floor prices
    println!("🎯 Create nod tokens with different floor prices");

    // Token 1: floor_price = 0.5 (above threshold, should not qualify)
    let msg = NodExecuteMsg::Submit {
        token_id: "nod_1".to_string(),
        owner: owner_addr.to_string(),
        extension: Box::new(SubmitExtension {
            entity: NodEntity {
                nod_id: "nod_1".to_string(),
                worldwide_day: 20250101,
                settlement_currency: coen_denom.clone(),
                symbolic_rate: Decimal::from_str("1.0").unwrap(),
                floor_rate: Decimal::from_str("0.5").unwrap(),
                nominal_price: Decimal::from_str("1.0").unwrap(),
                issuance_price: Decimal::from_str("0.3").unwrap(),
                gratis_load_minor: Uint128::new(100),
                floor_price: Decimal::from_str("0.5").unwrap(),
                state: State::Issued,
                owner: owner_addr.to_string(),
                qualified_at: None,
                is_touch: false,
            },
            created_at: None,
        }),
    };
    app.execute_contract(owner_addr.clone(), nod.address.clone(), &msg, &[])
        .unwrap();

    // Token 2: floor_price = 0.2 (below threshold, should qualify)
    let msg = NodExecuteMsg::Submit {
        token_id: "nod_2".to_string(),
        owner: owner_addr.to_string(),
        extension: Box::new(SubmitExtension {
            entity: NodEntity {
                nod_id: "nod_2".to_string(),
                worldwide_day: 20250101,
                settlement_currency: coen_denom.clone(),
                symbolic_rate: Decimal::from_str("1.0").unwrap(),
                floor_rate: Decimal::from_str("0.2").unwrap(),
                nominal_price: Decimal::from_str("1.0").unwrap(),
                issuance_price: Decimal::from_str("0.3").unwrap(),
                gratis_load_minor: Uint128::new(100),
                floor_price: Decimal::from_str("0.2").unwrap(),
                state: State::Issued,
                owner: owner_addr.to_string(),
                qualified_at: None,
                is_touch: false,
            },
            created_at: None,
        }),
    };
    app.execute_contract(owner_addr.clone(), nod.address.clone(), &msg, &[])
        .unwrap();

    // Token 3: floor_price = 0.4 (equal to a threshold, should qualify)
    let msg = NodExecuteMsg::Submit {
        token_id: "nod_3".to_string(),
        owner: owner_addr.to_string(),
        extension: Box::new(SubmitExtension {
            entity: NodEntity {
                nod_id: "nod_3".to_string(),
                worldwide_day: 20250101,
                settlement_currency: coen_denom.clone(),
                symbolic_rate: Decimal::from_str("1.0").unwrap(),
                floor_rate: Decimal::from_str("0.4").unwrap(),
                nominal_price: Decimal::from_str("1.0").unwrap(),
                issuance_price: Decimal::from_str("0.3").unwrap(),
                gratis_load_minor: Uint128::new(100),
                floor_price: Decimal::from_str("0.4").unwrap(),
                state: State::Issued,
                owner: owner_addr.to_string(),
                qualified_at: None,
                is_touch: false,
            },
            created_at: None,
        }),
    };
    app.execute_contract(owner_addr.clone(), nod.address.clone(), &msg, &[])
        .unwrap();

    // Verify initial state: all tokens should be Issued
    println!("✅ Verify initial state: all tokens are Issued");
    for token_id in ["nod_1", "nod_2", "nod_3"] {
        let query_msg = NodQueryMsg::NftInfo {
            token_id: token_id.to_string(),
        };
        let res: outbe_nft::msg::NftInfoResponse<nod::types::NodData> = app
            .wrap()
            .query_wasm_smart(nod.address.clone(), &query_msg)
            .unwrap();
        assert_eq!(res.extension.state, State::Issued);
        assert_eq!(res.extension.qualified_at, None);
        println!("  Token {}: State = {:?}", token_id, res.extension.state);
    }

    // Update price in oracle to 0.4 (this should trigger qualification)
    println!("💰 Update price in oracle to 0.4");
    let msg = PriceOracleExecuteMsg::UpdatePrice {
        token1: coen_denom.clone(),
        token2: usdc_denom.clone(),
        price: Decimal::from_str("0.4").unwrap(),
        open: Some(Decimal::from_str("0.38").unwrap()),
        high: Some(Decimal::from_str("0.42").unwrap()),
        low: Some(Decimal::from_str("0.37").unwrap()),
        close: Some(Decimal::from_str("0.4").unwrap()),
        volume: Some(Uint128::new(1200)),
    };
    let res = app
        .execute_contract(owner_addr.clone(), price_oracle.address.clone(), &msg, &[])
        .unwrap();

    println!("📊 Price update response events: {:?}", res.events);

    // Verify final state: tokens with floor_price <= 0.4 should be Qualified
    println!("✅ Verify final state after price update");

    // nod_1 (floor_price = 0.5) should still be Issued
    let query_msg = NodQueryMsg::NftInfo {
        token_id: "nod_1".to_string(),
    };
    let res: outbe_nft::msg::NftInfoResponse<nod::types::NodData> = app
        .wrap()
        .query_wasm_smart(nod.address.clone(), &query_msg)
        .unwrap();
    assert_eq!(res.extension.state, State::Issued);
    assert_eq!(res.extension.qualified_at, None);
    println!(
        "  Token nod_1 (floor_price=0.5): State = {:?} ✅",
        res.extension.state
    );

    // nod_2 (floor_price = 0.2) should be Qualified
    let query_msg = NodQueryMsg::NftInfo {
        token_id: "nod_2".to_string(),
    };
    let res: outbe_nft::msg::NftInfoResponse<nod::types::NodData> = app
        .wrap()
        .query_wasm_smart(nod.address.clone(), &query_msg)
        .unwrap();
    assert_eq!(res.extension.state, State::Qualified);
    assert!(res.extension.qualified_at.is_some());
    println!(
        "  Token nod_2 (floor_price=0.2): State = {:?} ✅",
        res.extension.state
    );

    // nod_3 (floor_price = 0.4) should be Qualified
    let query_msg = NodQueryMsg::NftInfo {
        token_id: "nod_3".to_string(),
    };
    let res: outbe_nft::msg::NftInfoResponse<nod::types::NodData> = app
        .wrap()
        .query_wasm_smart(nod.address.clone(), &query_msg)
        .unwrap();
    assert_eq!(res.extension.state, State::Qualified);
    assert!(res.extension.qualified_at.is_some());
    println!(
        "  Token nod_3 (floor_price=0.4): State = {:?} ✅",
        res.extension.state
    );

    println!(
        "🎉 Test completed successfully! Price oracle update triggered nod token qualification."
    );
}
