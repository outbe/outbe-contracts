#[cfg(test)]
use crate::contract::{execute, instantiate};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::query::{query, QueryMsg};
use crate::types::{DayType, PriceData, TokenPair};
use cosmwasm_std::testing::{mock_dependencies, mock_dependencies_with_balance, mock_env};
use cosmwasm_std::{from_json, Decimal, Env, MessageInfo, Timestamp};
use outbe_utils::denom::Denom;
use std::str::FromStr;

pub const CREATOR_ADDR: &str = "creator";
// pub const UNAUTHORIZED_ADDR: &str = "unauthorized";

pub const COEN: &str = "COEN";
pub const USDC: &str = "USDC";

fn get_default_instantiate_msg() -> InstantiateMsg {
    InstantiateMsg { creator: None }
}

fn add_default_token_pair(
    deps: &mut cosmwasm_std::OwnedDeps<
        cosmwasm_std::MemoryStorage,
        cosmwasm_std::testing::MockApi,
        cosmwasm_std::testing::MockQuerier,
    >,
    env: &Env,
    info: &MessageInfo,
) {
    let msg = ExecuteMsg::AddTokenPair {
        token1: Denom::Native(COEN.to_string()),
        token2: Denom::Native(USDC.to_string()),
    };
    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
}

#[test]
fn add_token_pair() {
    let mut deps = mock_dependencies();
    let msg = get_default_instantiate_msg();
    let info = MessageInfo {
        sender: deps.api.addr_make(CREATOR_ADDR),
        funds: vec![],
    };
    let env = mock_env();

    instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Add new token pair
    let msg = ExecuteMsg::AddTokenPair {
        token1: Denom::Native("ubtc".to_string()),
        token2: Denom::Native("ueth".to_string()),
    };
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(1, res.events.len());

    // Check if pair exists in query
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAllPairs {}).unwrap();
    let pairs: Vec<TokenPair> = from_json(&res).unwrap();
    assert_eq!(1, pairs.len());
}

#[test]
fn add_duplicate_pair_fails() {
    let mut deps = mock_dependencies();
    let msg = get_default_instantiate_msg();
    let info = MessageInfo {
        sender: deps.api.addr_make(CREATOR_ADDR),
        funds: vec![],
    };
    let env = mock_env();

    instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // First add the pair
    add_default_token_pair(&mut deps, &env, &info);

    // Try to add duplicate pair
    let msg = ExecuteMsg::AddTokenPair {
        token1: Denom::Native(COEN.to_string()),
        token2: Denom::Native(USDC.to_string()),
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    match err {
        ContractError::PairAlreadyExists { .. } => {}
        e => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn update_price() {
    let mut deps = mock_dependencies_with_balance(&[]);
    let msg = get_default_instantiate_msg();
    let info = MessageInfo {
        sender: deps.api.addr_make(CREATOR_ADDR),
        funds: vec![],
    };
    let env = mock_env();

    instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Add the token pair first
    add_default_token_pair(&mut deps, &env, &info);

    // Update price with new API
    let msg = ExecuteMsg::UpdatePrice {
        token1: Denom::Native(COEN.to_string()),
        token2: Denom::Native(USDC.to_string()),
        price: Decimal::from_str("2.0").unwrap(),
        open: Some(Decimal::from_str("1.8").unwrap()),
        high: Some(Decimal::from_str("2.1").unwrap()),
        low: Some(Decimal::from_str("1.7").unwrap()),
        close: Some(Decimal::from_str("2.0").unwrap()),
    };
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(1, res.events.len());

    // Check updated price
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetLatestPrice {
            token1: Denom::Native(COEN.to_string()),
            token2: Denom::Native(USDC.to_string()),
        },
    )
    .unwrap();
    let value: PriceData = from_json(&res).unwrap();
    assert_eq!(Decimal::from_str("2.0").unwrap(), value.price);
    assert_eq!(Some(Decimal::from_str("1.8").unwrap()), value.open);
}

// #[test]
// fn unauthorized_fails() {
//     let mut deps = mock_dependencies();
//     let msg = get_default_instantiate_msg();
//     let info = MessageInfo {
//         sender: deps.api.addr_make(CREATOR_ADDR),
//         funds: vec![],
// };
//     let env = mock_env();

//     instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

//     // Try to add pair with unauthorized user
//     let unauthorized_info = MessageInfo {
//         sender: deps.api.addr_make(UNAUTHORIZED_ADDR),
//         funds: vec![],
// };

//     let msg = ExecuteMsg::AddTokenPair {
//         token1: Denom::Native("ubtc".to_string()),
//         token2: Denom::Native("ueth".to_string()),
// };
//     let err = execute(deps.as_mut(), env, unauthorized_info, msg).unwrap_err();
//     print!("{}", err);

//     match err {
//         ContractError::Ownership(_) => {}
//         e => panic!("Unexpected error: {:?}", e),
// }
// }

#[test]
fn remove_token_pair() {
    let mut deps = mock_dependencies();
    let msg = get_default_instantiate_msg();
    let info = MessageInfo {
        sender: deps.api.addr_make(CREATOR_ADDR),
        funds: vec![],
    };
    let env = mock_env();

    instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Add a pair
    let msg = ExecuteMsg::AddTokenPair {
        token1: Denom::Native("ubtc".to_string()),
        token2: Denom::Native("ueth".to_string()),
    };
    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Remove the pair
    let msg = ExecuteMsg::RemoveTokenPair {
        token1: Denom::Native("ubtc".to_string()),
        token2: Denom::Native("ueth".to_string()),
    };
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(1, res.events.len());

    // Check pairs count
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAllPairs {}).unwrap();
    let pairs: Vec<TokenPair> = from_json(&res).unwrap();
    assert_eq!(0, pairs.len());
}

#[test]
fn query_price_history_with_valid_range() {
    let mut deps = mock_dependencies();
    let msg = get_default_instantiate_msg();
    let info = MessageInfo {
        sender: deps.api.addr_make(CREATOR_ADDR),
        funds: vec![],
    };
    let mut env = mock_env();

    instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Add the token pair first
    add_default_token_pair(&mut deps, &env, &info);

    // Update price multiple times with different timestamps
    env.block.time = Timestamp::from_seconds(1000);
    let msg = ExecuteMsg::UpdatePrice {
        token1: Denom::Native(COEN.to_string()),
        token2: Denom::Native(USDC.to_string()),
        price: Decimal::from_str("1.5").unwrap(),
        open: None,
        high: None,
        low: None,
        close: None,
    };
    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    env.block.time = Timestamp::from_seconds(2000);
    let msg = ExecuteMsg::UpdatePrice {
        token1: Denom::Native(COEN.to_string()),
        token2: Denom::Native(USDC.to_string()),
        price: Decimal::from_str("1.6").unwrap(),
        open: None,
        high: None,
        low: None,
        close: None,
    };
    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    env.block.time = Timestamp::from_seconds(3000);
    let msg = ExecuteMsg::UpdatePrice {
        token1: Denom::Native(COEN.to_string()),
        token2: Denom::Native(USDC.to_string()),
        price: Decimal::from_str("1.7").unwrap(),
        open: None,
        high: None,
        low: None,
        close: None,
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    // Query price history for the full range
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetPriceHistory {
            token1: Denom::Native(COEN.to_string()),
            token2: Denom::Native(USDC.to_string()),
            start_time: Timestamp::from_seconds(1000),
            end_time: Timestamp::from_seconds(3000),
        },
    )
    .unwrap();
    let history: Vec<PriceData> = from_json(&res).unwrap();
    assert_eq!(3, history.len());
    assert_eq!(Decimal::from_str("1.5").unwrap(), history[0].price);
    assert_eq!(Decimal::from_str("1.6").unwrap(), history[1].price);
    assert_eq!(Decimal::from_str("1.7").unwrap(), history[2].price);
}

#[test]
fn query_price_history_with_invalid_time_range() {
    let mut deps = mock_dependencies();
    let msg = get_default_instantiate_msg();
    let info = MessageInfo {
        sender: deps.api.addr_make(CREATOR_ADDR),
        funds: vec![],
    };
    let env = mock_env();

    instantiate(deps.as_mut(), env, info, msg).unwrap();

    // Query with start_time >= end_time
    let err = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetPriceHistory {
            token1: Denom::Native(COEN.to_string()),
            token2: Denom::Native(USDC.to_string()),
            start_time: Timestamp::from_seconds(2000),
            end_time: Timestamp::from_seconds(1000),
        },
    )
    .unwrap_err();
    assert!(err.to_string().contains("Invalid time range"));
}

#[test]
fn query_price_history_with_invalid_token_pair() {
    let mut deps = mock_dependencies();
    let msg = get_default_instantiate_msg();
    let info = MessageInfo {
        sender: deps.api.addr_make(CREATOR_ADDR),
        funds: vec![],
    };
    let env = mock_env();

    instantiate(deps.as_mut(), env, info, msg).unwrap();

    // Query with same token pair (invalid)
    let err = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetPriceHistory {
            token1: Denom::Native(COEN.to_string()),
            token2: Denom::Native(COEN.to_string()),
            start_time: Timestamp::from_seconds(1000),
            end_time: Timestamp::from_seconds(2000),
        },
    )
    .unwrap_err();
    assert!(err.to_string().contains("Invalid token pair"));
}

#[test]
fn query_price_history_with_empty_history() {
    let mut deps = mock_dependencies();
    let msg = get_default_instantiate_msg();
    let info = MessageInfo {
        sender: deps.api.addr_make(CREATOR_ADDR),
        funds: vec![],
    };
    let env = mock_env();

    instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Add a new token pair
    let msg = ExecuteMsg::AddTokenPair {
        token1: Denom::Native("BTC".to_string()),
        token2: Denom::Native("ETH".to_string()),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    // Query price history for new pair (should be empty)
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetPriceHistory {
            token1: Denom::Native("BTC".to_string()),
            token2: Denom::Native("ETH".to_string()),
            start_time: Timestamp::from_seconds(0),
            end_time: Timestamp::from_seconds(5000),
        },
    )
    .unwrap();
    let history: Vec<PriceData> = from_json(&res).unwrap();
    assert_eq!(0, history.len());
}

#[test]
fn query_price_history_with_partial_range() {
    let mut deps = mock_dependencies();
    let msg = get_default_instantiate_msg();
    let info = MessageInfo {
        sender: deps.api.addr_make(CREATOR_ADDR),
        funds: vec![],
    };
    let mut env = mock_env();

    instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Add the token pair first
    add_default_token_pair(&mut deps, &env, &info);

    // Update price multiple times with different timestamps
    env.block.time = Timestamp::from_seconds(1000);
    let msg = ExecuteMsg::UpdatePrice {
        token1: Denom::Native(COEN.to_string()),
        token2: Denom::Native(USDC.to_string()),
        price: Decimal::from_str("1.5").unwrap(),
        open: Some(Decimal::from_str("1.4").unwrap()),
        high: Some(Decimal::from_str("1.6").unwrap()),
        low: Some(Decimal::from_str("1.3").unwrap()),
        close: Some(Decimal::from_str("1.5").unwrap()),
    };
    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    env.block.time = Timestamp::from_seconds(2000);
    let msg = ExecuteMsg::UpdatePrice {
        token1: Denom::Native(COEN.to_string()),
        token2: Denom::Native(USDC.to_string()),
        price: Decimal::from_str("1.6").unwrap(),
        open: Some(Decimal::from_str("1.5").unwrap()),
        high: Some(Decimal::from_str("1.7").unwrap()),
        low: Some(Decimal::from_str("1.4").unwrap()),
        close: Some(Decimal::from_str("1.6").unwrap()),
    };
    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    env.block.time = Timestamp::from_seconds(3000);
    let msg = ExecuteMsg::UpdatePrice {
        token1: Denom::Native(COEN.to_string()),
        token2: Denom::Native(USDC.to_string()),
        price: Decimal::from_str("1.7").unwrap(),
        open: Some(Decimal::from_str("1.6").unwrap()),
        high: Some(Decimal::from_str("1.8").unwrap()),
        low: Some(Decimal::from_str("1.5").unwrap()),
        close: Some(Decimal::from_str("1.7").unwrap()),
    };
    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    env.block.time = Timestamp::from_seconds(4000);
    let msg = ExecuteMsg::UpdatePrice {
        token1: Denom::Native(COEN.to_string()),
        token2: Denom::Native(USDC.to_string()),
        price: Decimal::from_str("1.8").unwrap(),
        open: None,
        high: None,
        low: None,
        close: None,
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    // Query price history for partial range (middle two entries)
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetPriceHistory {
            token1: Denom::Native(COEN.to_string()),
            token2: Denom::Native(USDC.to_string()),
            start_time: Timestamp::from_seconds(1500),
            end_time: Timestamp::from_seconds(3500),
        },
    )
    .unwrap();
    let history: Vec<PriceData> = from_json(&res).unwrap();
    assert_eq!(2, history.len());
    assert_eq!(Decimal::from_str("1.6").unwrap(), history[0].price);
    assert_eq!(Decimal::from_str("1.7").unwrap(), history[1].price);

    // Verify OHLC data is preserved
    assert_eq!(Some(Decimal::from_str("1.5").unwrap()), history[0].open);
    assert_eq!(Some(Decimal::from_str("1.7").unwrap()), history[0].high);
    assert_eq!(Some(Decimal::from_str("1.4").unwrap()), history[0].low);
    assert_eq!(Some(Decimal::from_str("1.6").unwrap()), history[0].close);
}

// Day type tests
#[test]
fn test_day_type_initialization() {
    let mut deps = mock_dependencies();
    let msg = get_default_instantiate_msg();
    let info = MessageInfo {
        sender: deps.api.addr_make(CREATOR_ADDR),
        funds: vec![],
    };
    let env = mock_env();

    instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Add the token pair first
    add_default_token_pair(&mut deps, &env, &info);

    // Set the day type
    let msg = ExecuteMsg::SetDayType {
        token1: Denom::Native(COEN.to_string()),
        token2: Denom::Native(USDC.to_string()),
        day_type: DayType::Green,
    };
    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Check that day type is set correctly
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetDayType {
            token1: Denom::Native(COEN.to_string()),
            token2: Denom::Native(USDC.to_string()),
        },
    )
    .unwrap();
    let day_type: DayType = from_json(&res).unwrap();
    assert_eq!(DayType::Green, day_type);
}

#[test]
fn test_set_day_type_success() {
    let mut deps = mock_dependencies();
    let msg = get_default_instantiate_msg();
    let info = MessageInfo {
        sender: deps.api.addr_make(CREATOR_ADDR),
        funds: vec![],
    };
    let env = mock_env();

    instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Add the token pair first
    add_default_token_pair(&mut deps, &env, &info);

    // Set day type to Red
    let msg = ExecuteMsg::SetDayType {
        token1: Denom::Native(COEN.to_string()),
        token2: Denom::Native(USDC.to_string()),
        day_type: DayType::Red,
    };
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(1, res.events.len());
    assert_eq!("price-oracle::day_type_set", res.events[0].ty);

    // Verify day type was updated
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetDayType {
            token1: Denom::Native(COEN.to_string()),
            token2: Denom::Native(USDC.to_string()),
        },
    )
    .unwrap();
    let day_type: DayType = from_json(&res).unwrap();
    assert_eq!(DayType::Red, day_type);
}

// #[test]
// fn test_set_day_type_unauthorized() {
//     let mut deps = mock_dependencies();
//     let msg = get_default_instantiate_msg();
//     let info = MessageInfo {
//         sender: deps.api.addr_make(CREATOR_ADDR),
//         funds: vec![],
//     };
//     let env = mock_env();

//     instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

//     // Try to set day type with unauthorized user
//     let unauthorized_info = MessageInfo {
//         sender: deps.api.addr_make(UNAUTHORIZED_ADDR),
//         funds: vec![],
//     };
//     let msg = ExecuteMsg::SetDayType {
//         token1: Denom::Native(COEN.to_string()),
//         token2: Denom::Native(USDC.to_string()),
//         day_type: DayType::Red,
//     };
//     let err = execute(deps.as_mut(), env, unauthorized_info, msg).unwrap_err();
//     match err {
//         ContractError::Ownership(_) => {}
//         e => panic!("Unexpected error: {:?}", e),
//     }
// }

#[test]
fn test_set_day_type_invalid_pair() {
    let mut deps = mock_dependencies();
    let msg = get_default_instantiate_msg();
    let info = MessageInfo {
        sender: deps.api.addr_make(CREATOR_ADDR),
        funds: vec![],
    };
    let env = mock_env();

    instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Try to set day type with same token (invalid pair)
    let msg = ExecuteMsg::SetDayType {
        token1: Denom::Native(COEN.to_string()),
        token2: Denom::Native(COEN.to_string()),
        day_type: DayType::Red,
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    match err {
        ContractError::InvalidTokenPair {} => {}
        e => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn test_set_day_type_pair_not_found() {
    let mut deps = mock_dependencies();
    let msg = get_default_instantiate_msg();
    let info = MessageInfo {
        sender: deps.api.addr_make(CREATOR_ADDR),
        funds: vec![],
    };
    let env = mock_env();

    instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Try to set day type for non-existent pair
    let msg = ExecuteMsg::SetDayType {
        token1: Denom::Native("BTC".to_string()),
        token2: Denom::Native("ETH".to_string()),
        day_type: DayType::Red,
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    match err {
        ContractError::PairNotFound { .. } => {}
        e => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn test_get_day_type_pair_not_found() {
    let mut deps = mock_dependencies();
    let msg = get_default_instantiate_msg();
    let info = MessageInfo {
        sender: deps.api.addr_make(CREATOR_ADDR),
        funds: vec![],
    };
    let env = mock_env();

    instantiate(deps.as_mut(), env, info, msg).unwrap();

    // Try to query day type for non-existent pair
    let err = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetDayType {
            token1: Denom::Native("BTC".to_string()),
            token2: Denom::Native("ETH".to_string()),
        },
    )
    .unwrap_err();
    assert!(err.to_string().contains("Day type not found"));
}

#[test]
fn test_get_day_type_invalid_pair() {
    let mut deps = mock_dependencies();
    let msg = get_default_instantiate_msg();
    let info = MessageInfo {
        sender: deps.api.addr_make(CREATOR_ADDR),
        funds: vec![],
    };
    let env = mock_env();

    instantiate(deps.as_mut(), env, info, msg).unwrap();

    // Try to query day type with same token (invalid pair)
    let err = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetDayType {
            token1: Denom::Native(COEN.to_string()),
            token2: Denom::Native(COEN.to_string()),
        },
    )
    .unwrap_err();
    assert!(err.to_string().contains("Invalid token pair"));
}

#[test]
fn test_day_type_removed_with_pair() {
    let mut deps = mock_dependencies();
    let msg = get_default_instantiate_msg();
    let info = MessageInfo {
        sender: deps.api.addr_make(CREATOR_ADDR),
        funds: vec![],
    };
    let env = mock_env();

    instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Add a new pair
    let msg = ExecuteMsg::AddTokenPair {
        token1: Denom::Native("BTC".to_string()),
        token2: Denom::Native("ETH".to_string()),
    };
    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Set day type for the new pair
    let msg = ExecuteMsg::SetDayType {
        token1: Denom::Native("BTC".to_string()),
        token2: Denom::Native("ETH".to_string()),
        day_type: DayType::Green,
    };
    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Verify day type was set
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetDayType {
            token1: Denom::Native("BTC".to_string()),
            token2: Denom::Native("ETH".to_string()),
        },
    )
    .unwrap();
    let day_type: DayType = from_json(&res).unwrap();
    assert_eq!(DayType::Green, day_type);

    // Remove the pair
    let msg = ExecuteMsg::RemoveTokenPair {
        token1: Denom::Native("BTC".to_string()),
        token2: Denom::Native("ETH".to_string()),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    // Verify day type is also removed
    let err = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetDayType {
            token1: Denom::Native("BTC".to_string()),
            token2: Denom::Native("ETH".to_string()),
        },
    )
    .unwrap_err();
    assert!(err.to_string().contains("Day type not found"));
}
