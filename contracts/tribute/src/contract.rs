use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, MintExtension, TributeCollectionExtension,
};
use crate::types::{TributeConfig, TributeData, TributeNft};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Decimal, DepsMut, Env, Event, MessageInfo, Response, Uint128};
use outbe_nft::execute::assert_minter;
use outbe_nft::msg::CollectionInfoMsg;
use outbe_nft::state::{CollectionInfo, Cw721Config};
use outbe_utils::consts::DECIMAL_PLACES;

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
        symbolic_rate: msg.collection_info_extension.symbolic_rate,
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
        ExecuteMsg::BurnAll {} => execute_burn_all(deps, &env, &info),
        // todo remove only for day
        ExecuteMsg::BurnForDay { .. } => execute_burn_all(deps, &env, &info),

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
                symbolic_rate: data.symbolic_rate,
                native_token: data.native_token,
                price_oracle: data.price_oracle,
            },
        )?;
    }

    let response = Response::new().add_attribute("action", "update_collection_info");
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
        || entity.nominal_qty_minor == Uint128::zero()
        || entity.nominal_price_minor == Decimal::zero()
    {
        return Err(ContractError::WrongInput {});
    }

    let config = Cw721Config::<TributeData, TributeConfig>::default();
    let col_config = config.collection_config.load(deps.storage)?;

    // TODO here we need to check that given exchange_rate is in daily bounds
    // let exchange_rate: price_oracle::types::TokenPairPrice = deps.querier.query_wasm_smart(
    //     &col_config.price_oracle,
    //     &price_oracle::query::QueryMsg::GetPrice {},
    // )?;
    //

    let (nominal_qty, symbolic_load) = calc_sybolics(
        entity.settlement_amount_minor,
        entity.nominal_price_minor,
        col_config.symbolic_rate,
    );

    // create the token
    let data = TributeData {
        settlement_amount_minor: entity.settlement_amount_minor,
        settlement_currency: entity.settlement_currency,
        nominal_price_minor: entity.nominal_price_minor,
        nominal_qty_minor: nominal_qty,
        symbolic_load,
        worldwide_day: entity.worldwide_day,
        created_at: env.block.time,
    };

    let token = TributeNft {
        owner: owner_addr,
        token_uri,
        extension: data,
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
                .add_attribute("owner", owner),
        ))
}

fn calc_sybolics(
    settlement_amount: Uint128,
    exchange_rate: Decimal,
    symbolic_rate: Decimal,
) -> (Uint128, Uint128) {
    let settlement_value_dec = Decimal::from_atomics(settlement_amount, DECIMAL_PLACES).unwrap();
    let nominal_qty = settlement_value_dec / exchange_rate;
    let symbolic_load = nominal_qty * symbolic_rate / (Decimal::one() + symbolic_rate);

    println!("settlement_value: {}", settlement_amount);
    println!("exchange_rate: {}", exchange_rate);
    println!("symbolic_rate: {}", symbolic_rate);
    println!("nominal_qty: {}", nominal_qty);
    println!("symbolic_load: {}", symbolic_load);
    (nominal_qty.atomics(), symbolic_load.atomics())
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
fn execute_burn_all(
    deps: DepsMut,
    _env: &Env,
    info: &MessageInfo,
) -> Result<Response, ContractError> {
    let config = Cw721Config::<TributeData, TributeConfig>::default();
    // TODO verify ownership
    // let token = config.nft_info.load(deps.storage, &token_id)?;
    // check_can_send(deps.as_ref(), env, info.sender.as_str(), &token)?;

    config.clean_tokens(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "tribute::burn_all")
        .add_event(
            Event::new("tribute::burn_all").add_attribute("sender", info.sender.to_string()),
        ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    match msg {
        MigrateMsg::Migrate {} => Ok(Response::new()),
    }
}

#[cfg(test)]
mod tests {
    use crate::contract::{calc_sybolics, execute_burn_all, instantiate};
    use crate::msg::{InstantiateMsg, TributeCollectionExtension};
    use crate::types::{TributeConfig, TributeData, TributeNft};
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env, MockApi};
    use cosmwasm_std::{Addr, Decimal, Storage, Timestamp, Uint128};
    use outbe_nft::state::Cw721Config;
    use outbe_utils::denom::{Currency, Denom};
    use std::str::FromStr;

    #[test]
    fn test_symbolics_calc() {
        let (nominal, load) = calc_sybolics(
            Uint128::new(500_000000000000000000u128),
            Decimal::from_str("0.2").unwrap(),
            Decimal::from_str("0.08").unwrap(),
        );
        assert_eq!(nominal, Uint128::new(2500000000000000000000u128));
        assert_eq!(load, Uint128::new(185185185185185185185u128));
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
                    symbolic_rate: Decimal::from_str("0.08").unwrap(),
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
        let res = execute_burn_all(deps.as_mut(), &env, &info).unwrap();

        // Verify tokens were burned
        assert_eq!(config.token_count(&deps.storage).unwrap(), 0);
        assert!(res
            .attributes
            .iter()
            .any(|attr| attr.key == "action" && attr.value == "tribute::burn_all"));
    }

    fn create_test_token(storage: &mut dyn Storage, token_id: &str) {
        let config = Cw721Config::<TributeData, TributeConfig>::default();
        let token = TributeNft {
            owner: Addr::unchecked("owner"),
            token_uri: None,
            extension: TributeData {
                settlement_amount_minor: Uint128::new(100),
                settlement_currency: Denom::Fiat(Currency::Usd),
                nominal_price_minor: Decimal::one(),
                nominal_qty_minor: Uint128::new(100),
                symbolic_load: Uint128::new(10),
                worldwide_day: 1,
                created_at: Timestamp::from_seconds(1000),
            },
        };
        config.nft_info.save(storage, token_id, &token).unwrap();
        config.increment_tokens(storage).unwrap();
    }
}
