use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, SubmitExtension};
use crate::types::{NodConfig, NodData, NodNft};
use cosmwasm_std::{
    to_json_binary, Decimal, DepsMut, Env, Event, MessageInfo, QueryRequest, Response, Uint128,
    WasmMsg, WasmQuery,
};
use outbe_nft::error::Cw721ContractError;
use outbe_nft::state::{CollectionInfo, Cw721Config};
use outbe_utils::denom::Denom;

// External contract message types
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum PriceOracleQueryMsg {
    GetPrice {},
}

#[cw_serde]
pub struct TokenPairPrice {
    pub token1: Denom,
    pub token2: Denom,
    pub price: Decimal,
    pub day_type: String,
}

#[cw_serde]
pub enum TokenMinerExecuteMsg {
    Mine {
        recipient: String,
        amount: Uint128,
        token_type: TokenType,
    },
}

#[cw_serde]
pub enum TokenType {
    Gratis,
    Promis,
}

const CONTRACT_NAME: &str = "outbe.net:nod";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let cfg = NodConfig {
        price_oracle_contract: deps
            .api
            .addr_validate(&msg.collection_info_extension.price_oracle_contract)?,
        token_miner_contract: deps
            .api
            .addr_validate(&msg.collection_info_extension.token_miner_contract)?,
    };
    let collection_info = CollectionInfo {
        name: msg.name,
        symbol: msg.symbol,
        updated_at: env.block.time,
    };
    let config = Cw721Config::<NodData, NodConfig>::default();
    config.collection_config.save(deps.storage, &cfg)?;
    config
        .collection_info
        .save(deps.storage, &collection_info)?;

    let minter = msg
        .minter
        .clone()
        .unwrap_or_else(|| info.sender.to_string());
    outbe_nft::execute::initialize_minter(deps.storage, deps.api, Some(&minter))?;

    let creator = msg
        .creator
        .clone()
        .unwrap_or_else(|| info.sender.to_string());
    outbe_nft::execute::initialize_creator(deps.storage, deps.api, Some(&creator))?;

    Ok(Response::new()
        .add_attribute("action", "nod::instantiate")
        .add_attribute("minter", minter)
        .add_attribute("creator", creator))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Submit {
            token_id,
            owner,
            extension,
        } => execute_submit(deps, &env, &info, token_id, owner, *extension),
        ExecuteMsg::Burn { token_id } => execute_burn(deps, &env, &info, token_id),
        ExecuteMsg::Claim { token_id } => execute_claim(deps, &env, &info, token_id),
        ExecuteMsg::UpdateSettings {
            price_oracle_contract,
            token_miner_contract,
        } => execute_update_settings(
            deps,
            &env,
            &info,
            price_oracle_contract,
            token_miner_contract,
        ),
        ExecuteMsg::BurnAll {} => execute_burn_all(deps, &env, &info),
    }
}

#[allow(clippy::too_many_arguments)]
fn execute_submit(
    deps: DepsMut,
    env: &Env,
    _info: &MessageInfo,
    token_id: String,
    owner: String,
    extension: SubmitExtension,
) -> Result<Response, ContractError> {
    // TODO uncomment after demo
    // outbe_nft::execute::assert_minter(deps.storage, &info.sender)?;

    let owner_addr = deps.api.addr_validate(&owner)?;
    let entity = extension.entity;

    let data = NodData {
        nod_id: entity.nod_id.clone(),
        settlement_currency: entity.settlement_currency.clone(),
        symbolic_rate: entity.symbolic_rate,
        floor_rate: entity.floor_rate,
        nominal_price_minor: entity.nominal_price_minor,
        issuance_price_minor: entity.issuance_price_minor,
        gratis_load_minor: entity.gratis_load_minor,
        floor_price_minor: entity.floor_price_minor,
        state: entity.state.clone(),
        owner: entity.owner.clone(),
        issued_at: extension.created_at.unwrap_or(env.block.time),
        qualified_at: entity.qualified_at,
    };
    let token = NodNft {
        owner: owner_addr,
        token_uri: None, // todo populate
        extension: data,
    };

    let config = Cw721Config::<NodData, NodConfig>::default();
    config
        .nft_info
        .update(deps.storage, &token_id, |old| match old {
            Some(_) => Err(Cw721ContractError::Claimed {}),
            None => Ok(token),
        })?;
    config.increment_tokens(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "nod::submit")
        .add_attribute("token_id", token_id)
        .add_attribute("owner", owner))
}

fn execute_burn(
    deps: DepsMut,
    _env: &Env,
    info: &MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    let config = Cw721Config::<NodData, NodConfig>::default();
    config.nft_info.remove(deps.storage, &token_id)?;
    config.decrement_tokens(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "nod::burn")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("token_id", token_id))
}

fn execute_burn_all(
    deps: DepsMut,
    _env: &Env,
    info: &MessageInfo,
) -> Result<Response, ContractError> {
    let config = Cw721Config::<NodData, NodConfig>::default();
    // TODO verify ownership
    // let token = config.nft_info.load(deps.storage, &token_id)?;
    // check_can_send(deps.as_ref(), env, info.sender.as_str(), &token)?;

    config.nft_info.clear(deps.storage);
    config.token_count.save(deps.storage, &0u64)?;

    Ok(Response::new()
        .add_attribute("action", "nod::burn_all")
        .add_event(Event::new("nod::burn_all").add_attribute("sender", info.sender.to_string())))
}

fn execute_claim(
    deps: DepsMut,
    _env: &Env,
    info: &MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    let config = Cw721Config::<NodData, NodConfig>::default();
    let nod_config = config.collection_config.load(deps.storage)?;

    // Load the NFT data
    let nft = config.nft_info.load(deps.storage, &token_id)?;

    // Check if the caller is the owner of the NFT
    if nft.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // Get the nominal_minor_rate (gratis amount) from the NFT
    let gratis_amount = nft.extension.gratis_load_minor;

    // Query the price oracle to get the current price
    let price_query = QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: nod_config.price_oracle_contract.to_string(),
        msg: to_json_binary(&PriceOracleQueryMsg::GetPrice {})?,
    });

    let price_response: TokenPairPrice = deps.querier.query(&price_query)?;

    // Compare floor_minor_price with oracle price (Get price as decimal 18.)
    if nft.extension.floor_price_minor <= price_response.price.atomics() {
        // Call token-miner to mine gratis tokens
        let mine_msg = TokenMinerExecuteMsg::Mine {
            recipient: info.sender.to_string(),
            amount: gratis_amount,
            token_type: TokenType::Gratis,
        };

        let wasm_msg = WasmMsg::Execute {
            contract_addr: nod_config.token_miner_contract.to_string(),
            msg: to_json_binary(&mine_msg)?,
            funds: vec![],
        };

        Ok(Response::new()
            .add_message(wasm_msg)
            .add_attribute("action", "nod::claim")
            .add_attribute("token_id", token_id)
            .add_attribute("claimer", info.sender.to_string())
            .add_attribute("gratis_amount", gratis_amount.to_string())
            .add_attribute("floor_price", nft.extension.floor_price_minor.to_string())
            .add_attribute("oracle_price", price_response.price.to_string()))
    } else {
        Err(ContractError::ClaimConditionNotMet {})
    }
}

fn execute_update_settings(
    deps: DepsMut,
    _env: &Env,
    info: &MessageInfo,
    price_oracle_contract: Option<String>,
    token_miner_contract: Option<String>,
) -> Result<Response, ContractError> {
    let config = Cw721Config::<NodData, NodConfig>::default();
    let mut nod_config = config.collection_config.load(deps.storage)?;

    // TODO: Add proper access control (only admin/creator should be able to update)
    // For now, we'll use a simple check - you might want to implement proper admin functionality

    let mut response = Response::new()
        .add_attribute("action", "nod::update_contracts")
        .add_attribute("updater", info.sender.to_string());

    // Update price oracle contract if provided
    if let Some(oracle_addr) = price_oracle_contract {
        let validated_addr = deps.api.addr_validate(&oracle_addr)?;
        nod_config.price_oracle_contract = validated_addr;
        response = response.add_attribute("new_price_oracle", oracle_addr);
    }

    // Update token miner contract if provided
    if let Some(miner_addr) = token_miner_contract {
        let validated_addr = deps.api.addr_validate(&miner_addr)?;
        nod_config.token_miner_contract = validated_addr;
        response = response.add_attribute("new_token_miner", miner_addr);
    }

    // Save the updated configuration
    config.collection_config.save(deps.storage, &nod_config)?;

    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new())
}
