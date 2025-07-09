use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, TeeSetup, ZkProof};
use crate::state::{Config, CONFIG, OWNER, USED_CU_HASHES, USED_IDS};
use crate::types::TributeInputPayload;
use cosmwasm_std::{entry_point, Addr, DepsMut, Empty, Env, Event, MessageInfo, Response, Storage};
use cw_ownable::Action;

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
    _info: MessageInfo,
    tribute_input: TributeInputPayload,
    _zk_proof: ZkProof,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let _tribute_address = config
        .tribute_address
        .ok_or(ContractError::NotInitialized {})?;

    let tribute = tee_obfuscate(tribute_input)?;
    update_used_state(deps.storage, tribute)?;

    // todo issue tribute

    Ok(Response::new()
        .add_attribute("action", "tribute-factory::offer_insecure")
        .add_event(Event::new("tribute-factory::offer_insecure")))
}

fn tee_obfuscate(tribute_input: TributeInputPayload) -> Result<TributeInputPayload, ContractError> {
    // TODO implement tee obfuscation

    Ok(tribute_input)
}

fn update_used_state(
    storage: &mut dyn Storage,
    tribute: TributeInputPayload,
) -> Result<Empty, ContractError> {
    let tribute_draft_id = tribute.tribute_draft_id.to_hex();

    USED_IDS.update(storage, tribute_draft_id, |old| match old {
        Some(_) => Err(ContractError::IdAlreadyExists {}),
        None => Ok(Empty::default()),
    })?;

    for cu_hash in tribute.cu_hashes {
        let cu_hash_hex = cu_hash.to_hex();
        USED_CU_HASHES.update(storage, cu_hash_hex, |old| match old {
            Some(_) => Err(ContractError::CUAlreadyExists {}),
            None => Ok(Empty::default()),
        })?;
    }
    Ok(Empty::default())
}
