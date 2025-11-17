use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, MintExtension, TributeCollectionExtension,
};
use crate::types::{TributeConfig, TributeData, TributeNft};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Decimal, DepsMut, Env, Event, MessageInfo, Order, Response, Uint128};
use cw_storage_plus::Index;
use outbe_nft::execute::assert_minter;
use outbe_nft::msg::CollectionInfoMsg;
use outbe_nft::state::{CollectionInfo, Cw721Config};
use outbe_utils::consts::DECIMAL_PLACES;
use outbe_utils::date::WorldwideDay;

const CONTRACT_NAME: &str = "outbe.net:tribute";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let cfg = TributeConfig {
        native_token: msg.collection_info_extension.native_token.clone(),
        price_oracle: msg.collection_info_extension.price_oracle.clone(),
    };

    let collection_info = CollectionInfo {
        name: msg.name,
        symbol: msg.symbol,
        updated_at: env.block.time,
    };

    let config = Cw721Config::<TributeData, TributeConfig>::default();
    config.collection_config.save(deps.storage, &cfg)?;
    config
        .collection_info
        .save(deps.storage, &collection_info)?;

    // ---- set minter and creator ----
    // use info.sender if None is passed
    let minter: &str = match msg.minter.as_deref() {
        Some(minter) => minter,
        None => info.sender.as_str(),
    };
    outbe_nft::execute::initialize_minter(deps.storage, deps.api, Some(minter))?;

    // use info.sender if None is passed
    let burner: &str = match msg.burner.as_deref() {
        Some(burner) => burner,
        None => info.sender.as_str(),
    };
    outbe_nft::execute::initialize_burner(deps.storage, deps.api, Some(burner))?;

    // use info.sender if None is passed
    let creator: &str = match msg.creator.as_deref() {
        Some(creator) => creator,
        None => info.sender.as_str(),
    };
    outbe_nft::execute::initialize_creator(deps.storage, deps.api, Some(creator))?;

    Ok(Response::default()
        .add_attribute("action", "tribute::instantiate")
        .add_event(
            Event::new("tribute::instantiate")
                .add_attribute("minter", minter)
                .add_attribute("creator", creator),
        ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    match msg {
        MigrateMsg::Migrate {} => Ok(Response::new()),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mint {
            token_id,
            owner,
            token_uri,
            extension,
        } => execute_mint(deps, &env, &info, token_id, owner, token_uri, *extension),
        ExecuteMsg::Burn { token_id } => execute_burn(deps, &env, &info, token_id),
        #[cfg(feature = "demo")]
        ExecuteMsg::BurnAll { batch_size } => execute_burn_all(deps, &env, &info, batch_size),
        ExecuteMsg::BurnForDay { date } => execute_burn_for_day(deps, &env, &info, date),

        ExecuteMsg::UpdateMinterOwnership(action) => Ok(
            outbe_nft::execute::update_minter_ownership(deps, &env, &info, action)?,
        ),
        ExecuteMsg::UpdateCreatorOwnership(action) => Ok(
            outbe_nft::execute::update_creator_ownership(deps, &env, &info, action)?,
        ),
        ExecuteMsg::UpdateBurnerOwnership(action) => Ok(
            outbe_nft::execute::update_burner_ownership(deps, &env, &info, action)?,
        ),
        ExecuteMsg::UpdateCollectionInfo { collection_info } => {
            update_collection_info(deps, collection_info)
        }
    }
}

pub fn update_collection_info(
    deps: DepsMut,
    msg: CollectionInfoMsg<Option<TributeCollectionExtension>>,
) -> Result<Response, ContractError> {
    let config = Cw721Config::<TributeData, TributeConfig>::default();

    let mut collection_info = config.collection_info.load(deps.storage)?;
    if msg.name.is_some() {
        collection_info.name = msg.name.unwrap();
    }
    if msg.symbol.is_some() {
        collection_info.symbol = msg.symbol.unwrap();
    }

    if msg.extension.is_some() {
        let data = msg.extension.unwrap();
        config.collection_config.save(
            deps.storage,
            &TributeConfig {
                native_token: data.native_token,
                price_oracle: data.price_oracle,
            },
        )?;
    }

    let response = Response::new().add_attribute("action", "tribute::update_collection_info");
    Ok(response)
}

#[allow(clippy::too_many_arguments)]
fn execute_mint(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    token_id: String,
    owner: String,
    token_uri: Option<String>,
    extension: MintExtension,
) -> Result<Response, ContractError> {
    assert_minter(deps.storage, &info.sender)?;
    // validate owner
    let owner_addr = deps.api.addr_validate(&owner)?;

    let entity = extension.data;
    if entity.tribute_id != token_id || entity.owner != owner {
        return Err(ContractError::WrongInput {});
    }

    if entity.settlement_amount_minor == Uint128::zero()
        || entity.nominal_amount_minor == Uint128::zero()
        || entity.nominal_price == Decimal::zero()
    {
        return Err(ContractError::WrongInput {});
    }

    let config = Cw721Config::<TributeData, TributeConfig>::default();

    // TODO here we need to check that given exchange_rate is in daily bounds
    // let exchange_rate: price_oracle::types::TokenPairPrice = deps.querier.query_wasm_smart(
    //     &col_config.price_oracle,
    //     &price_oracle::query::QueryMsg::GetPrice {},
    // )?;
    //

    let nominal_amount = calc_nominal_amount(entity.settlement_amount_minor, entity.nominal_price);

    // create the token
    let data = TributeData {
        settlement_amount_minor: entity.settlement_amount_minor,
        settlement_currency: entity.settlement_currency,
        nominal_price: entity.nominal_price,
        nominal_amount_minor: nominal_amount,
        worldwide_day: entity.worldwide_day,
        created_at: env.block.time,
    };

    let token = TributeNft {
        owner: owner_addr,
        token_uri,
        extension: data.clone(),
    };

    config
        .nft_info
        .update(deps.storage, &token_id, |old| match old {
            Some(_) => Err(ContractError::AlreadyExists {}),
            None => Ok(token),
        })?;

    config.increment_tokens(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "tribute::mint")
        .add_event(
            Event::new("tribute::mint")
                .add_attribute("token_id", token_id)
                .add_attribute("owner", owner)
                .add_attribute("worldwide_day", data.worldwide_day.to_string())
                .add_attribute(
                    "nominal_amount_minor",
                    data.nominal_amount_minor.to_string(),
                )
                .add_attribute("nominal_price", data.nominal_price.to_string())
                .add_attribute(
                    "settlement_amount_minor",
                    data.settlement_amount_minor.to_string(),
                )
                .add_attribute("settlement_currency", data.settlement_currency.to_string()),
        ))
}

fn calc_nominal_amount(settlement_amount: Uint128, exchange_rate: Decimal) -> Uint128 {
    let settlement_value_dec = Decimal::from_atomics(settlement_amount, DECIMAL_PLACES).unwrap();
    let nominal_amount = settlement_value_dec / exchange_rate;

    println!("settlement_value: {}", settlement_amount);
    println!("exchange_rate: {}", exchange_rate);
    println!("nominal_amount: {}", nominal_amount);

    nominal_amount.atomics()
}

fn execute_burn(
    deps: DepsMut,
    _env: &Env,
    info: &MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    let config = Cw721Config::<TributeData, TributeConfig>::default();
    // TODO verify ownership
    // let token = config.nft_info.load(deps.storage, &token_id)?;
    // check_can_send(deps.as_ref(), env, info.sender.as_str(), &token)?;

    config.nft_info.remove(deps.storage, &token_id)?;
    config.decrement_tokens(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "tribute::burn")
        .add_event(
            Event::new("tribute::burn")
                .add_attribute("sender", info.sender.to_string())
                .add_attribute("token_id", token_id),
        ))
}

#[cfg(feature = "demo")]
fn execute_burn_all(
    deps: DepsMut,
    _env: &Env,
    info: &MessageInfo,
    batch_size: Option<usize>,
) -> Result<Response, ContractError> {
    let config = Cw721Config::<TributeData, TributeConfig>::default();

    config.clean_tokens(deps.storage, batch_size)?;

    Ok(Response::new()
        .add_attribute("action", "tribute::burn_all")
        .add_event(
            Event::new("tribute::burn_all").add_attribute("sender", info.sender.to_string()),
        ))
}

fn execute_burn_for_day(
    deps: DepsMut,
    _env: &Env,
    info: &MessageInfo,
    date: WorldwideDay,
) -> Result<Response, ContractError> {
    // Temporary disable ownership check because it's should be accepted when transferring
    // outbe_nft::execute::assert_burner(deps.storage, &info.sender)?;
    let config = Cw721Config::<TributeData, TributeConfig>::default();

    // Collect token IDs that match the specified date
    let tokens_to_burn: Vec<String> = config
        .nft_info
        .range(deps.storage, None, None, Order::Ascending)
        .filter_map(|item| {
            if let Ok((token_id, token_info)) = item {
                if token_info.extension.worldwide_day == date {
                    Some(token_id)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    let burned_count = tokens_to_burn.len();

    // Remove the tokens and update indexes
    for token_id in tokens_to_burn {
        if let Ok(token_info) = config.nft_info.load(deps.storage, &token_id) {
            // Remove from indexes
            config
                .nft_info
                .idx
                .owner
                .remove(deps.storage, token_id.as_bytes(), &token_info)?;
        }
        // Remove the token
        config.nft_info.remove(deps.storage, &token_id)?;
        // Decrement token count
        config.decrement_tokens(deps.storage)?;
    }

    Ok(Response::new()
        .add_attribute("action", "tribute::burn_for_day")
        .add_event(
            Event::new("tribute::burn_for_day")
                .add_attribute("sender", info.sender.to_string())
                .add_attribute("date", date.to_string())
                .add_attribute("burned_count", burned_count.to_string()),
        ))
}

#[cfg(test)]
mod tests {
    use crate::contract::{
        calc_nominal_amount, execute_burn_all, execute_burn_for_day, instantiate,
    };
    use crate::msg::{InstantiateMsg, TributeCollectionExtension};
    use crate::types::{TributeConfig, TributeData, TributeNft};
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env, MockApi};
    use cosmwasm_std::{Addr, Decimal, Storage, Timestamp, Uint128};
    use outbe_nft::state::Cw721Config;
    use outbe_utils::date::WorldwideDay;
    use outbe_utils::denom::{Currency, Denom};
    use std::str::FromStr;

    #[test]
    fn test_symbolics_calc() {
        let nominal = calc_nominal_amount(
            Uint128::new(500_000000000000000000u128),
            Decimal::from_str("0.2").unwrap(),
        );
        assert_eq!(nominal, Uint128::new(2500000000000000000000u128));
    }

    #[test]
    fn test_burn_all() {
        let api = MockApi::default();
        let owner_addr = api.addr_make("owner");
        let oracle_addr = api.addr_make("oracle");

        let mut deps = mock_dependencies();
        let info = message_info(&owner_addr, &[]);
        let env = mock_env();

        instantiate(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            InstantiateMsg {
                name: "Test".to_string(),
                symbol: "TEST".to_string(),
                minter: None,
                creator: None,
                burner: None,
                collection_info_extension: TributeCollectionExtension {
                    native_token: Denom::Native("coen".to_string()),
                    price_oracle: oracle_addr,
                },
            },
        )
        .unwrap();

        // Create some test tokens
        create_test_token(deps.as_mut().storage, "token1");
        create_test_token(deps.as_mut().storage, "token2");

        let config = Cw721Config::<TributeData, TributeConfig>::default();
        assert_eq!(config.token_count(&deps.storage).unwrap(), 2);

        // Execute burn all
        let res = execute_burn_all(deps.as_mut(), &env, &info, None).unwrap();

        // Verify tokens were burned
        assert_eq!(config.token_count(&deps.storage).unwrap(), 0);
        assert!(res
            .attributes
            .iter()
            .any(|attr| attr.key == "action" && attr.value == "tribute::burn_all"));
    }

    #[test]
    fn test_burn_for_day() {
        let api = MockApi::default();
        let owner_addr = api.addr_make("owner");
        let oracle_addr = api.addr_make("oracle");

        let mut deps = mock_dependencies();
        let info = message_info(&owner_addr, &[]);
        let env = mock_env();

        instantiate(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            InstantiateMsg {
                name: "Test".to_string(),
                symbol: "TEST".to_string(),
                minter: None,
                creator: None,
                burner: None,
                collection_info_extension: TributeCollectionExtension {
                    native_token: Denom::Native("coen".to_string()),
                    price_oracle: oracle_addr,
                },
            },
        )
        .unwrap();

        // Create test tokens with different dates
        create_test_token_with_day(deps.as_mut().storage, "token1", 20250101);
        create_test_token_with_day(deps.as_mut().storage, "token2", 20250101);
        create_test_token_with_day(deps.as_mut().storage, "token3", 20250102);
        create_test_token_with_day(deps.as_mut().storage, "token4", 20250103);

        let config = Cw721Config::<TributeData, TributeConfig>::default();
        assert_eq!(config.token_count(&deps.storage).unwrap(), 4);

        // Execute burn for day 1
        let res = execute_burn_for_day(deps.as_mut(), &env, &info, 20250101).unwrap();

        // Verify only tokens from day 1 were burned (2 tokens)
        assert_eq!(config.token_count(&deps.storage).unwrap(), 2);

        // Verify the response attributes
        assert!(res
            .attributes
            .iter()
            .any(|attr| attr.key == "action" && attr.value == "tribute::burn_for_day"));

        // Verify the event attributes
        assert!(!res.events.is_empty());
        let event = &res.events[0];
        assert!(event
            .attributes
            .iter()
            .any(|attr| attr.key == "date" && attr.value == "20250101"));
        assert!(event
            .attributes
            .iter()
            .any(|attr| attr.key == "burned_count" && attr.value == "2"));

        // Verify remaining tokens are from day 2 and 3
        assert!(config.nft_info.load(&deps.storage, "token1").is_err());
        assert!(config.nft_info.load(&deps.storage, "token2").is_err());
        assert!(config.nft_info.load(&deps.storage, "token3").is_ok());
        assert!(config.nft_info.load(&deps.storage, "token4").is_ok());
    }

    fn create_test_token(storage: &mut dyn Storage, token_id: &str) {
        let config = Cw721Config::<TributeData, TributeConfig>::default();
        let token = TributeNft {
            owner: Addr::unchecked("owner"),
            token_uri: None,
            extension: TributeData {
                settlement_amount_minor: Uint128::new(100),
                settlement_currency: Denom::Fiat(Currency::Usd),
                nominal_price: Decimal::one(),
                nominal_amount_minor: Uint128::new(100),
                worldwide_day: 1,
                created_at: Timestamp::from_seconds(1000),
            },
        };
        config.nft_info.save(storage, token_id, &token).unwrap();
        config.increment_tokens(storage).unwrap();
    }

    fn create_test_token_with_day(storage: &mut dyn Storage, token_id: &str, day: WorldwideDay) {
        let config = Cw721Config::<TributeData, TributeConfig>::default();
        let token = TributeNft {
            owner: Addr::unchecked("owner"),
            token_uri: None,
            extension: TributeData {
                settlement_amount_minor: Uint128::new(100),
                settlement_currency: Denom::Fiat(Currency::Usd),
                nominal_price: Decimal::one(),
                nominal_amount_minor: Uint128::new(100),
                worldwide_day: day,
                created_at: Timestamp::from_seconds(1000),
            },
        };
        config.nft_info.save(storage, token_id, &token).unwrap();
        config.increment_tokens(storage).unwrap();
    }
}
