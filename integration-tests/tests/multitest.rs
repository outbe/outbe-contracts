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
                    nominal_qty_minor: Uint128::from(100000000u32),
                    settlement_amount_minor: Uint128::from(100000000u32),
                    worldwide_day: normalize_to_date(&app.block_info().time),
                    nominal_price_minor: Decimal::one(),
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

    println!("📦 Deploy Vector");
    let vector = deploy_vector(&mut app, config.owner_addr.clone());

    println!("📦 Deploy Metadosis");
    let metadosis = deploy_metadosis(
        &mut app,
        config.owner_addr.clone(),
        tribute.address.clone(),
        nod.address.clone(),
        token_allocator.address.clone(),
        vector.address.clone(),
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
                    nominal_qty_minor: Uint128::from(10_000000000000000000u128),
                    worldwide_day: normalize_to_date(&app.block_info().time),
                    nominal_price_minor: Decimal::from_str("0.5").unwrap(),
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
                    nominal_qty_minor: Uint128::from(5_000000000000000000u128),
                    worldwide_day: normalize_to_date(&app.block_info().time),
                    nominal_price_minor: Decimal::from_str("3").unwrap(),
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

    let response: tribute::query::DailyTributesResponse = app
        .wrap()
        .query_wasm_smart(
            tribute.address.clone(),
            &QueryMsg::DailyTributes {
                date: normalize_to_date(&app.block_info().time),
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

    // TODO uncomment when finalize lysis
    // assert_eq!(response.tokens.len(), 2);
    //
    // println!("🔬 Lysis 2");
    // app.execute_contract(
    //     config.owner_addr.clone(),
    //     metadosis.address.clone(),
    //     &metadosis::msg::ExecuteMsg::Execute { run_date: None },
    //     &[],
    // )
    // .unwrap();
    //
    // let response: outbe_nft::msg::TokensResponse = app
    //     .wrap()
    //     .query_wasm_smart(
    //         nod.address.clone(),
    //         &QueryMsg::AllTokens {
    //             start_after: None,
    //             limit: None,
    //             query_order: None,
    //         },
    //     )
    //     .unwrap();
    //
    // assert_eq!(response.tokens.len(), 2);
    //
    // println!("🔬 Lysis 3");
    // app.execute_contract(
    //     config.owner_addr.clone(),
    //     metadosis.address.clone(),
    //     &metadosis::msg::ExecuteMsg::Execute { run_date: None },
    //     &[],
    // )
    // .unwrap();
    //
    // let response: outbe_nft::msg::TokensResponse = app
    //     .wrap()
    //     .query_wasm_smart(
    //         nod.address.clone(),
    //         &QueryMsg::AllTokens {
    //             start_after: None,
    //             limit: None,
    //             query_order: None,
    //         },
    //     )
    //     .unwrap();
    //
    // assert_eq!(
    //     response.tokens.len(),
    //     2,
    //     "No new nods because there were no tributes"
    // );
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

fn deploy_random_oracle(app: &mut App, owner: Addr) -> DeployedContract {
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
fn deploy_metadosis(
    app: &mut App,
    owner: Addr,
    tribute: Addr,
    nod: Addr,
    token_allocator: Addr,
    vector: Addr,
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
        vector: Some(vector),
        tribute: Some(tribute),
        nod: Some(nod),
        token_allocator: Some(token_allocator),
        price_oracle: Some(price_oracle),
        random_oracle: Some(random_oracle),
        deficit: Decimal::from_str("0.08").unwrap(),
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

fn deploy_price_oracle(app: &mut App, owner: Addr) -> DeployedContract {
    use price_oracle::contract::{execute, instantiate};
    use price_oracle::msg::InstantiateMsg;
    use price_oracle::query::query;

    let code = ContractWrapper::new(execute, instantiate, query);
    let code_id = app.store_code(Box::new(code));

    let instantiate_msg = InstantiateMsg {
        creator: None,
        vwap_window_seconds: Some(300),
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
