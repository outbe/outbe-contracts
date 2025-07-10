use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, TeeSetup, TributeMintData, TributeMintExtension, TributeMsg,
    ZkProof,
};
use crate::state::{Config, CONFIG, OWNER, UNUSED_TOKEN_ID, USED_CU_HASHES, USED_IDS};
use crate::types::TributeInputPayload;
use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Decimal, DepsMut, Empty, Env, Event, MessageInfo, Response,
    Storage, WasmMsg,
};
use cw_ownable::Action;
use outbe_utils::amount_utils::normalize_amount;
use outbe_utils::denom::Denom;

const CONTRACT_NAME: &str = "outbe.net:tribute-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(
        deps.storage,
        &Config {
            tribute_address: msg.tribute_address,
            tee_config: None, // todo impl, in scope of tee
        },
    )?;

    UNUSED_TOKEN_ID.save(deps.storage, &0)?;

    // ---- set owner ----
    let owner = msg.owner.unwrap_or(info.sender);
    OWNER.initialize_owner(deps.storage, deps.api, Some(owner.as_str()))?;

    Ok(Response::new()
        .add_attribute("action", "tribute-factory::instantiate")
        .add_event(Event::new("tribute-factory::instantiate").add_attribute("owner", owner)))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig {
            new_owner,
            new_tribute_address,
            new_tee_config,
        } => execute_update_config(
            deps,
            env,
            info,
            new_owner,
            new_tribute_address,
            new_tee_config,
        ),
        ExecuteMsg::Offer { .. } => {
            unimplemented!()
        }
        ExecuteMsg::OfferInsecure {
            tribute_input,
            zk_proof,
        } => execute_offer_insecure(deps, env, info, tribute_input, zk_proof),
    }
}

fn execute_update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    new_owner: Option<Addr>,
    new_tribute_address: Option<Addr>,
    new_tee_config: Option<TeeSetup>,
) -> Result<Response, ContractError> {
    OWNER.assert_owner(deps.storage, &info.sender)?;

    if new_tribute_address.is_some() || new_tee_config.is_some() {
        let mut config = CONFIG.load(deps.storage)?;
        if let Some(new_tribute_address) = new_tribute_address {
            config.tribute_address = Some(new_tribute_address)
        }
        if let Some(_new_tee_config) = new_tee_config {
            config.tee_config = None // todo impl tee
        }
        CONFIG.save(deps.storage, &config)?;
    }

    if let Some(new_owner) = new_owner {
        OWNER.update_ownership(
            deps,
            &env.block,
            &info.sender,
            Action::TransferOwnership {
                new_owner: new_owner.to_string(),
                expiry: None,
            },
        )?;
    }

    Ok(Response::new()
        .add_attribute("action", "tribute-factory::update_config")
        .add_event(Event::new("tribute-factory::update_config")))
}

fn execute_offer_insecure(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    tribute_input: TributeInputPayload,
    _zk_proof: ZkProof,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let tribute_address = config
        .tribute_address
        .ok_or(ContractError::NotInitialized {})?;

    let tribute = tee_obfuscate(tribute_input)?;
    update_used_state(deps.storage, &tribute)?;

    let tribute_id =
        UNUSED_TOKEN_ID.update(deps.storage, |old| Ok::<u64, ContractError>(old + 1))?;

    let tribute_owner = info.sender;

    let settlement_amount = normalize_amount(
        tribute.settlement_base_amount,
        tribute.settlement_atto_amount,
    )?;
    let settlement_qty = normalize_amount(tribute.nominal_base_qty, tribute.nominal_atto_qty)?;
    let tribute_price = settlement_amount / settlement_qty;

    let msg = WasmMsg::Execute {
        contract_addr: tribute_address.to_string(),
        msg: to_json_binary(&TributeMsg::Mint {
            token_id: tribute_id.to_string(),
            owner: tribute_owner.to_string(),
            token_uri: None,
            extension: Box::new(TributeMintExtension {
                data: TributeMintData {
                    tribute_id: tribute_id.to_string(),
                    worldwide_day: tribute.worldwide_day,
                    owner: tribute_owner.to_string(),
                    settlement_amount_minor: settlement_amount,
                    settlement_currency: Denom::Native(tribute.settlement_currency), // TODO use native
                    nominal_qty_minor: settlement_qty,
                    tribute_price_minor: Decimal::new(tribute_price),
                },
            }),
        })?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "tribute-factory::offer_insecure")
        .add_event(Event::new("tribute-factory::offer_insecure")))
}

fn tee_obfuscate(tribute_input: TributeInputPayload) -> Result<TributeInputPayload, ContractError> {
    // TODO implement tee obfuscation

    Ok(tribute_input)
}

fn update_used_state(
    storage: &mut dyn Storage,
    tribute: &TributeInputPayload,
) -> Result<Empty, ContractError> {
    let tribute_draft_id = tribute.tribute_draft_id.to_hex();

    USED_IDS.update(storage, tribute_draft_id, |old| match old {
        Some(_) => Err(ContractError::IdAlreadyExists {}),
        None => Ok(Empty::default()),
    })?;

    for cu_hash in tribute.cu_hashes.clone() {
        let cu_hash_hex = cu_hash.to_hex();
        USED_CU_HASHES.update(storage, cu_hash_hex, |old| match old {
            Some(_) => Err(ContractError::CUAlreadyExists {}),
            None => Ok(Empty::default()),
        })?;
    }
    Ok(Empty::default())
}
