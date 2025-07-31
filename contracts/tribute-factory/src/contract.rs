use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, TeeSetup, TributeMintData, TributeMintExtension, TributeMsg,
    ZkProof,
};
use crate::state::{Config, CONFIG, OWNER, UNUSED_TOKEN_ID, USED_CU_HASHES, USED_TRIBUTE_IDS};
use crate::types::TributeInputPayload;
use blake3::Hasher;
use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Decimal, DepsMut, Empty, Env, Event, HexBinary, MessageInfo,
    Response, Storage, WasmMsg,
};
use cw_ownable::Action;
use outbe_utils::amount_utils::normalize_amount;
use outbe_utils::date::{iso_to_ts, Iso8601Date};
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
            tribute_owner_l1,
        } => execute_offer_insecure(deps, env, info, tribute_input, zk_proof, tribute_owner_l1),
        ExecuteMsg::BurnAll {} => execute_burn_all(deps, env, info),
    }
}

fn execute_burn_all(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
) -> Result<Response, ContractError> {
    USED_CU_HASHES.clear(deps.storage);
    USED_TRIBUTE_IDS.clear(deps.storage);
    Ok(Response::new())
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
    tribute_owner_l1: Option<Addr>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let tribute_address = config
        .tribute_address
        .ok_or(ContractError::NotInitialized {})?;

    // validate
    if tribute_input.cu_hashes.is_empty() {
        return Err(ContractError::CUEmpty {});
    }

    //TODO change it to info.sender
    let tribute_owner = tribute_owner_l1.unwrap_or(info.sender.clone());

    let timestamp_date = iso_to_ts(&tribute_input.worldwide_day)?;
    // let timestamp_date = _env.block.time.seconds();

    let tribute = tee_obfuscate(tribute_input)?;
    update_used_state(deps.storage, &tribute)?;

    let tribute_id =
        UNUSED_TOKEN_ID.update(deps.storage, |old| Ok::<u64, ContractError>(old + 1))?;

    let settlement_amount = normalize_amount(
        tribute.settlement_base_amount,
        tribute.settlement_atto_amount,
    )?;
    let settlement_qty = normalize_amount(tribute.nominal_base_qty, tribute.nominal_atto_qty)?;
    // todo use safe convertion
    let tribute_price = Decimal::from_atomics(settlement_amount, 18).unwrap()
        / Decimal::from_atomics(settlement_qty, 18).unwrap();

    println!("settlement_amount {}", settlement_amount);
    println!("settlement_qty {}", settlement_qty);
    println!("tribute_price {}", tribute_price);

    let msg = WasmMsg::Execute {
        contract_addr: tribute_address.to_string(),
        msg: to_json_binary(&TributeMsg::Mint {
            token_id: tribute_id.to_string(),
            owner: tribute_owner.to_string(),
            token_uri: None,
            extension: Box::new(TributeMintExtension {
                data: TributeMintData {
                    tribute_id: tribute_id.to_string(),
                    worldwide_day: timestamp_date,
                    owner: tribute_owner.to_string(),
                    settlement_amount_minor: settlement_amount,
                    settlement_currency: Denom::Native(tribute.settlement_currency), // TODO use native
                    nominal_qty_minor: settlement_qty,
                    tribute_price_minor: tribute_price,
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
    let tribute_draft_id = generate_tribute_draft_id_hash(&tribute.owner, &tribute.worldwide_day);

    // Validate that provided draft ID matches tribute_draft_id
    if tribute.tribute_draft_id != tribute_draft_id {
        return Err(ContractError::InvalidDraftId {});
    }

    USED_TRIBUTE_IDS.update(storage, tribute_draft_id.to_string(), |old| match old {
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

pub fn generate_tribute_draft_id_hash(owner: &String, worldwide_day: &Iso8601Date) -> HexBinary {
    let mut hasher = Hasher::new();
    hasher.update(owner.as_bytes());
    hasher.update(worldwide_day.as_bytes());
    let hash_bytes: [u8; 32] = hasher.finalize().into();
    HexBinary::from(hash_bytes.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::ZkProofPublicData;
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::HexBinary;
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};
    use std::str::FromStr;

    fn tribute_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            tribute::contract::execute,
            tribute::contract::instantiate,
            tribute::query::query,
        );
        Box::new(contract)
    }

    #[test]
    fn test_mint_tribute_flow() {
        let mut app = App::default();
        let owner = app.api().addr_make("owner");
        let sender = app.api().addr_make("sender");
        let oracle = app.api().addr_make("oracle");

        println!("Deploy tribute-factory contract code");
        let factory_code = ContractWrapper::new(execute, instantiate, crate::query::query);
        let factory_code_id = app.store_code(Box::new(factory_code));

        // Instantiate tribute-factory contract
        let factory_addr = app
            .instantiate_contract(
                factory_code_id,
                owner.clone(),
                &InstantiateMsg {
                    owner: Some(owner.clone()),
                    tee_config: None,
                    tribute_address: None,
                    zk_config: None,
                },
                &[],
                "tribute-factory",
                None,
            )
            .unwrap();

        println!("Deploy tribute contract code");
        let tribute_code_id = app.store_code(tribute_contract());

        let tribute_addr = app
            .instantiate_contract(
                tribute_code_id,
                owner.clone(),
                &tribute::msg::InstantiateMsg {
                    name: "tribute".to_string(),
                    symbol: "tt".to_string(),
                    collection_info_extension: tribute::msg::TributeCollectionExtension {
                        symbolic_rate: Decimal::from_str("0.8").unwrap(),
                        native_token: Denom::Native("coen".to_string()),
                        price_oracle: oracle.clone(),
                    },
                    minter: Some(factory_addr.to_string()),
                    burner: None,
                    creator: None,
                },
                &[],
                "mock-tribute",
                None,
            )
            .unwrap();

        println!("Update tribute address");
        app.execute_contract(
            owner.clone(),
            factory_addr.clone(),
            &ExecuteMsg::UpdateConfig {
                new_tribute_address: Some(tribute_addr.clone()),
                new_owner: None,
                new_tee_config: None,
            },
            &[],
        )
        .unwrap();

        // Prepare inputs for execution
        let owner = sender.to_string();
        let worldwide_day = "2022-03-22".to_string();
        let cu_hash_1 = [11; 32];
        let cu_hash_2 = [22; 32];
        let tribute_input = TributeInputPayload {
            tribute_draft_id: generate_tribute_draft_id_hash(&owner, &worldwide_day),
            cu_hashes: vec![HexBinary::from(cu_hash_1), HexBinary::from(cu_hash_2)],
            worldwide_day,
            settlement_currency: "usd".to_string(),
            settlement_base_amount: 500, // 500 USD
            settlement_atto_amount: 0,
            nominal_base_qty: 1000,
            nominal_atto_qty: 0,
            owner,
        };

        // Execute the insecure offer
        app.execute_contract(
            sender.clone(),
            factory_addr.clone(),
            &ExecuteMsg::OfferInsecure {
                tribute_input: tribute_input.clone(),
                zk_proof: ZkProof {
                    proof: Default::default(),
                    public_data: ZkProofPublicData {
                        public_key: Default::default(),
                        merkle_root: Default::default(),
                    },
                    verification_key: Default::default(),
                },
                tribute_owner_l1: None,
            },
            &[],
        )
        .unwrap();

        // --- Assertions ---
        // todo add assertions
    }

    #[test]
    fn test_unique_tribute_draft_id() {
        let mut deps = mock_dependencies();
        let owner = "user1".to_string();
        let worldwide_day = "2022-03-22".to_string();
        println!(
            "id {}",
            generate_tribute_draft_id_hash(&owner, &worldwide_day)
        );

        let tribute = TributeInputPayload {
            tribute_draft_id: generate_tribute_draft_id_hash(&owner, &worldwide_day),
            cu_hashes: vec![HexBinary::from([11; 32])],
            worldwide_day,
            settlement_currency: "usd".to_string(),
            settlement_base_amount: 500,
            settlement_atto_amount: 0,
            nominal_base_qty: 1000,
            nominal_atto_qty: 0,
            owner,
        };

        // first call
        update_used_state(deps.as_mut().storage, &tribute).unwrap();

        // second call -  IdAlreadyExists
        let err = update_used_state(deps.as_mut().storage, &tribute).unwrap_err();
        assert!(matches!(err, ContractError::IdAlreadyExists {}));
    }

    #[test]
    fn test_unique_cu_hash() {
        let mut deps = cosmwasm_std::testing::mock_dependencies();
        let owner = "user1".to_string();
        let worldwide_day = "2022-03-22".to_string();

        let cu_hash = HexBinary::from([42; 32]);

        let tribute1 = TributeInputPayload {
            tribute_draft_id: generate_tribute_draft_id_hash(&owner, &worldwide_day),
            cu_hashes: vec![cu_hash.clone()],
            worldwide_day,
            settlement_currency: "usd".to_string(),
            settlement_base_amount: 500,
            settlement_atto_amount: 0,
            nominal_base_qty: 1000,
            nominal_atto_qty: 0,
            owner,
        };

        // Change worldwide_day && tribute_draft_id
        let mut tribute2 = tribute1.clone();
        tribute2.worldwide_day = "2022-03-23".to_string();
        tribute2.tribute_draft_id =
            generate_tribute_draft_id_hash(&tribute2.owner, &tribute2.worldwide_day);

        // first call
        update_used_state(deps.as_mut().storage, &tribute1).unwrap();

        // second call -  CUAlreadyExists
        let err = update_used_state(deps.as_mut().storage, &tribute2).unwrap_err();
        assert!(matches!(err, ContractError::CUAlreadyExists {}));
    }

    #[test]
    fn test_invalid_tribute_draft_id() {
        let mut deps = mock_dependencies();
        let tribute = TributeInputPayload {
            tribute_draft_id: HexBinary::from([42; 32]), // incorrect
            cu_hashes: vec![HexBinary::from([1; 32])],
            worldwide_day: "2022-03-22".to_string(),
            settlement_currency: "usd".to_string(),
            settlement_base_amount: 100,
            settlement_atto_amount: 0,
            nominal_base_qty: 1000,
            nominal_atto_qty: 0,
            owner: "user1".to_string(),
        };

        let err = update_used_state(deps.as_mut().storage, &tribute).unwrap_err();
        assert!(matches!(err, ContractError::InvalidDraftId {}));
    }
}
