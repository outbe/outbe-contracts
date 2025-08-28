use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, TeeSetup, TributeMintData, TributeMintExtension, TributeMsg,
    ZkProof,
};
use crate::state::{Config, TeeConfig, CONFIG, OWNER, USED_CU_HASHES, USED_TRIBUTE_IDS};
use crate::types::TributeInputPayload;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Decimal, DepsMut, Empty, Env, Event, HexBinary, MessageInfo,
    Response, Storage, WasmMsg,
};
use curve25519_dalek::{MontgomeryPoint, Scalar};
use cw_ownable::Action;
use hkdf::Hkdf;
use outbe_utils::amount_utils::normalize_amount;
use outbe_utils::date::{iso_to_days, iso_to_ts, normalize_to_date, Iso8601Date};
use outbe_utils::denom::Denom;
use outbe_utils::{gen_compound_hash, gen_hash, Base58Binary};
use sha2::Sha256;

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

    // Validate TEE config if provided
    let mut cft: Option<TeeConfig> = None;
    if let Some(ref tee_setup) = msg.tee_config {
        let pubkey = validate_tee_config(tee_setup)?;
        cft = Some(TeeConfig {
            private_key: tee_setup.private_key.clone(),
            public_key: pubkey,
            salt: tee_setup.salt.clone(),
        });
    }

    CONFIG.save(
        deps.storage,
        &Config {
            tribute_address: msg.tribute_address,
            tee_config: cft,
        },
    )?;

    // ---- set owner ----
    let owner = msg.owner.unwrap_or(info.sender);
    OWNER.initialize_owner(deps.storage, deps.api, Some(owner.as_str()))?;

    Ok(Response::new()
        .add_attribute("action", "tribute-factory::instantiate")
        .add_event(Event::new("tribute-factory::instantiate").add_attribute("owner", owner)))
}

fn validate_tee_config(tee_setup: &TeeSetup) -> Result<Base58Binary, ContractError> {
    // Validate salt length (32 bytes recommended)
    if tee_setup.salt.len() != 32 {
        return Err(ContractError::InvalidSalt {});
    }

    let private_key_array: [u8; 32] = tee_setup
        .private_key
        .as_slice()
        .try_into()
        .map_err(|_| ContractError::InvalidKey {})?;

    // Generate public key from private key
    let private_key_scalar = Scalar::from_bytes_mod_order(private_key_array);
    let derived_public_key_point =
        curve25519_dalek::constants::X25519_BASEPOINT * private_key_scalar;
    let derived_public_key_bytes = derived_public_key_point.to_bytes();

    Ok(Base58Binary::from(derived_public_key_bytes))
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
        ExecuteMsg::Offer {
            cipher_text,
            nonce,
            ephemeral_pubkey,
            zk_proof,
            #[cfg(feature = "demo")]
            tribute_owner_l1,
        } => execute_offer(
            deps,
            env,
            info,
            cipher_text,
            nonce,
            ephemeral_pubkey,
            zk_proof,
            #[cfg(feature = "demo")]
            tribute_owner_l1,
        ),
        #[cfg(feature = "demo")]
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

#[allow(clippy::too_many_arguments)]
fn execute_offer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cipher_text: Base58Binary,
    nonce: Base58Binary,
    ephemeral_pubkey: Base58Binary,
    zk_proof: ZkProof,
    #[cfg(feature = "demo")] tribute_owner_l1: Option<Addr>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let tee_config = config.tee_config.ok_or(ContractError::NotInitialized {})?;

    // Decrypt the tribute input using ECDHE
    let tribute_input =
        decrypt_tribute_input(&cipher_text, &nonce, &ephemeral_pubkey, &tee_config)?;

    // Process the decrypted tribute input (same logic as OfferInsecure)
    execute_offer_logic(
        deps,
        env,
        info,
        tribute_input,
        zk_proof,
        #[cfg(feature = "demo")]
        tribute_owner_l1,
    )
}

pub(crate) fn decrypt_tribute_input(
    cipher_text: &Base58Binary,
    nonce: &Base58Binary,
    ephemeral_pubkey: &Base58Binary,
    tee_config: &TeeConfig,
) -> Result<TributeInputPayload, ContractError> {
    let private_key_array: [u8; 32] = tee_config
        .private_key
        .as_slice()
        .try_into()
        .map_err(|_| ContractError::InvalidKey {})?;
    let ephemeral_pubkey_array: [u8; 32] = ephemeral_pubkey
        .as_slice()
        .try_into()
        .map_err(|_| ContractError::InvalidKey {})?;
    let nonce_array: [u8; 12] = nonce
        .as_slice()
        .try_into()
        .map_err(|_| ContractError::InvalidNonce {})?;

    let private_key = Scalar::from_bytes_mod_order(private_key_array);
    let ephemeral_public_key = MontgomeryPoint(ephemeral_pubkey_array);

    // Perform ECDH to get shared secret
    let shared_secret = ephemeral_public_key * private_key;

    // Use HKDF to derive an encryption key from shared secret and salt
    let hk = Hkdf::<Sha256>::new(Some(tee_config.salt.as_slice()), &shared_secret.to_bytes());
    let mut encryption_key = [0u8; 32];
    hk.expand(b"tribute-factory-encryption", &mut encryption_key)
        .map_err(|_| ContractError::DecryptionFailed {})?;

    // Use derived key for ChaCha20Poly1305
    let cipher = ChaCha20Poly1305::new((&encryption_key).into());

    // Decrypt the data
    let decrypted_bytes = cipher
        .decrypt(&Nonce::from(nonce_array), cipher_text.as_slice())
        .map_err(|_| ContractError::DecryptionFailed {})?;

    // Deserialize the decrypted data
    let tribute_input: TributeInputPayload =
        cosmwasm_std::from_json(&decrypted_bytes).map_err(|_| ContractError::InvalidPayload {})?;

    Ok(tribute_input)
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
        if let Some(new_tee_config) = new_tee_config {
            let pubkey = validate_tee_config(&new_tee_config)?;
            config.tee_config = Some(TeeConfig {
                private_key: new_tee_config.private_key,
                public_key: pubkey,
                salt: new_tee_config.salt,
            })
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

#[cfg(feature = "demo")]
fn execute_offer_insecure(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    tribute_input: TributeInputPayload,
    zk_proof: ZkProof,
    tribute_owner_l1: Option<Addr>,
) -> Result<Response, ContractError> {
    execute_offer_logic(deps, env, info, tribute_input, zk_proof, tribute_owner_l1)
}

fn execute_offer_logic(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    tribute_input: TributeInputPayload,
    _zk_proof: ZkProof,
    #[cfg(feature = "demo")] tribute_owner_l1: Option<Addr>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let tribute_address = config
        .tribute_address
        .ok_or(ContractError::NotInitialized {})?;

    // validate
    if tribute_input.cu_hashes.is_empty() {
        return Err(ContractError::CUEmpty {});
    }

    let tribute_owner = {
        #[cfg(feature = "demo")]
        {
            tribute_owner_l1.unwrap_or(info.sender.clone())
        }
        #[cfg(not(feature = "demo"))]
        {
            info.sender.clone()
        }
    };

    let timestamp_date = iso_to_ts(&tribute_input.worldwide_day)?;
    let today = normalize_to_date(&_env.block.time);
    if timestamp_date < today {
        return Err(ContractError::ClosedOfferWindow {});
    }
    // let timestamp_date = _env.block.time.seconds();

    let tribute = tee_obfuscate(tribute_input.clone())?;
    update_used_state(deps.storage, &tribute)?;

    let tribute_id = generate_tribute_id(
        &tribute.tribute_draft_id,
        &tribute_owner,
        &tribute_input.worldwide_day,
    );

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
                    nominal_price_minor: tribute_price,
                },
            }),
        })?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "tribute-factory::offer")
        .add_event(Event::new("tribute-factory::offer")))
}

fn tee_obfuscate(tribute_input: TributeInputPayload) -> Result<TributeInputPayload, ContractError> {
    // TODO implement tee obfuscation

    Ok(tribute_input)
}

fn update_used_state(
    storage: &mut dyn Storage,
    tribute: &TributeInputPayload,
) -> Result<Empty, ContractError> {
    // NB: temporary disable validation for demo
    #[cfg(not(feature = "demo"))]
    {
        let tribute_draft_id = crate::contract::generate_tribute_draft_id_hash(
            &tribute.owner,
            &tribute.worldwide_day,
        )?;
        // Validate that provided draft ID matches tribute_draft_id
        if tribute.tribute_draft_id != tribute_draft_id {
            return Err(ContractError::InvalidDraftId {});
        }
    }

    USED_TRIBUTE_IDS.update(
        storage,
        tribute.tribute_draft_id.to_base58(),
        |old| match old {
            Some(_) => Err(ContractError::IdAlreadyExists {}),
            None => Ok(Empty::default()),
        },
    )?;

    for cu_hash in tribute.cu_hashes.clone() {
        USED_CU_HASHES.update(storage, cu_hash.to_base58(), |old| match old {
            Some(_) => Err(ContractError::CUAlreadyExists {}),
            None => Ok(Empty::default()),
        })?;
    }
    Ok(Empty::default())
}

pub fn generate_tribute_draft_id_hash(
    owner: &Base58Binary,
    worldwide_day: &Iso8601Date,
) -> Result<Base58Binary, ContractError> {
    let days = iso_to_days(worldwide_day)?;
    let hex_bin = gen_hash(vec![owner.as_slice(), days.to_le_bytes().as_slice()]);
    Ok(Base58Binary::from(hex_bin.as_slice()))
}

fn generate_tribute_id(
    token_id: &Base58Binary,
    owner: &Addr,
    worldwide_day: &Iso8601Date,
) -> HexBinary {
    gen_compound_hash(
        Some("tribute-factory:tribute_id"),
        vec![
            token_id.as_slice(),
            owner.as_bytes(),
            worldwide_day.as_bytes(),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::ZkProofPublicData;
    use crate::test_ecdhe::generate_keypair;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{Uint128, Uint64};
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
        let owner = Base58Binary::from(sender.as_bytes());
        let worldwide_day = "2025-03-22".to_string();
        let cu_hash_1 = [11; 32];
        let cu_hash_2 = [22; 32];
        let tribute_input = TributeInputPayload {
            tribute_draft_id: generate_tribute_draft_id_hash(&owner, &worldwide_day).unwrap(),
            cu_hashes: vec![Base58Binary::from(cu_hash_1), Base58Binary::from(cu_hash_2)],
            worldwide_day,
            settlement_currency: "usd".to_string(),
            settlement_base_amount: Uint64::new(500), // 500 USD
            settlement_atto_amount: Uint128::zero(),
            nominal_base_qty: Uint64::new(1000),
            nominal_atto_qty: Uint128::zero(),
            owner,
        };

        // Execute the insecure offer
        #[cfg(feature = "demo")]
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
        let owner = Base58Binary::from("user1".as_bytes());
        let worldwide_day = "2025-03-22".to_string();
        println!(
            "id {:?}",
            generate_tribute_draft_id_hash(&owner, &worldwide_day)
        );

        let tribute = TributeInputPayload {
            tribute_draft_id: generate_tribute_draft_id_hash(&owner, &worldwide_day).unwrap(),
            cu_hashes: vec![Base58Binary::from([11; 32])],
            worldwide_day,
            settlement_currency: "usd".to_string(),
            settlement_base_amount: Uint64::new(500),
            settlement_atto_amount: Uint128::zero(),
            nominal_base_qty: Uint64::new(1000),
            nominal_atto_qty: Uint128::zero(),
            owner,
        };

        // first call
        update_used_state(deps.as_mut().storage, &tribute).unwrap();

        // second call - IdAlreadyExists
        let err = update_used_state(deps.as_mut().storage, &tribute).unwrap_err();
        assert!(matches!(err, ContractError::IdAlreadyExists {}));
    }
    #[test]
    fn test_tribute_draft_id() {
        let owner =
            Base58Binary::from_base58("5HpHagT65TDzv1PH4D1wkmPxqHL5vTMzMmPMDqqAqxnwfnXF").unwrap();
        let worldwide_day = "2025-01-01".to_string();
        let result1 = generate_tribute_draft_id_hash(&owner, &worldwide_day).unwrap();
        let result2 = generate_tribute_draft_id_hash(&owner, &worldwide_day).unwrap();

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_unique_cu_hash() {
        let mut deps = mock_dependencies();
        let owner = Base58Binary::from("user1".as_bytes());
        let worldwide_day = "2025-05-22".to_string();

        let cu_hash = Base58Binary::from([42; 32]);

        let tribute1 = TributeInputPayload {
            tribute_draft_id: generate_tribute_draft_id_hash(&owner, &worldwide_day).unwrap(),
            cu_hashes: vec![cu_hash.clone()],
            worldwide_day,
            settlement_currency: "usd".to_string(),
            settlement_base_amount: Uint64::new(500),
            settlement_atto_amount: Uint128::zero(),
            nominal_base_qty: Uint64::new(1000),
            nominal_atto_qty: Uint128::zero(),
            owner,
        };

        // Change worldwide_day && tribute_draft_id
        let mut tribute2 = tribute1.clone();
        tribute2.worldwide_day = "2025-03-23".to_string();
        tribute2.tribute_draft_id =
            generate_tribute_draft_id_hash(&tribute2.owner, &tribute2.worldwide_day).unwrap();

        // first call
        update_used_state(deps.as_mut().storage, &tribute1).unwrap();

        // second call - CUAlreadyExists
        let err = update_used_state(deps.as_mut().storage, &tribute2).unwrap_err();
        assert!(matches!(err, ContractError::CUAlreadyExists {}));
    }

    #[test]
    #[ignore] // tmp disable and wait for poseidon hashing
    fn test_invalid_tribute_draft_id() {
        let mut deps = mock_dependencies();
        let tribute = TributeInputPayload {
            tribute_draft_id: Base58Binary::from([42; 32]), // incorrect
            cu_hashes: vec![Base58Binary::from([1; 32])],
            worldwide_day: "2025-03-22".to_string(),
            settlement_currency: "usd".to_string(),
            settlement_base_amount: Uint64::new(100),
            settlement_atto_amount: Uint128::zero(),
            nominal_base_qty: Uint64::new(1000),
            nominal_atto_qty: Uint128::zero(),
            owner: Base58Binary::from("user1".as_bytes()),
        };

        let err = update_used_state(deps.as_mut().storage, &tribute).unwrap_err();
        assert!(matches!(err, ContractError::InvalidDraftId {}));
    }

    #[test]
    fn test_decrypt_tribute_input_with_hkdf() {
        use rand::rngs::OsRng;
        use rand::RngCore;

        // Generate contract keypair
        let mut private_key_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut private_key_bytes);
        let private_key_scalar = Scalar::from_bytes_mod_order(private_key_bytes);
        let public_key_point = curve25519_dalek::constants::X25519_BASEPOINT * private_key_scalar;
        let public_key_bytes = public_key_point.to_bytes();

        // Generate salt
        let mut salt_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut salt_bytes);

        // Create test tribute input
        let tribute_input = TributeInputPayload {
            tribute_draft_id: Base58Binary::from([42u8; 32]),
            cu_hashes: vec![Base58Binary::from([1u8; 32]), Base58Binary::from([2u8; 32])],
            worldwide_day: "2025-08-27".to_string(),
            settlement_currency: "usd".to_string(),
            settlement_base_amount: Uint64::new(1000),
            settlement_atto_amount: Uint128::zero(),
            nominal_base_qty: Uint64::new(500),
            nominal_atto_qty: Uint128::zero(),
            owner: Base58Binary::from("test_owner".as_bytes()),
        };

        // Encrypt tribute input (client side simulation)
        let mut ephemeral_private_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut ephemeral_private_bytes);
        let ephemeral_private_scalar = Scalar::from_bytes_mod_order(ephemeral_private_bytes);
        let ephemeral_public_point =
            curve25519_dalek::constants::X25519_BASEPOINT * ephemeral_private_scalar;
        let ephemeral_public_bytes = ephemeral_public_point.to_bytes();

        // Perform ECDH
        let contract_public_point = MontgomeryPoint(public_key_bytes);
        let shared_secret = contract_public_point * ephemeral_private_scalar;

        // Use HKDF to derive encryption key
        let hk = Hkdf::<Sha256>::new(Some(&salt_bytes), &shared_secret.to_bytes());
        let mut encryption_key = [0u8; 32];
        hk.expand(b"tribute-factory-encryption", &mut encryption_key)
            .unwrap();

        // Serialize and encrypt
        let plaintext = to_json_binary(&tribute_input).unwrap().to_vec();
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);

        let cipher = ChaCha20Poly1305::new((&encryption_key).into());
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher.encrypt(nonce, plaintext.as_ref()).unwrap();

        // Test decryption
        let tee_config = TeeConfig {
            private_key: Base58Binary::from(private_key_bytes),
            public_key: Base58Binary::from(public_key_bytes),
            salt: Base58Binary::from(salt_bytes),
        };

        let decrypted_input = decrypt_tribute_input(
            &Base58Binary::from(ciphertext),
            &Base58Binary::from(nonce_bytes),
            &Base58Binary::from(ephemeral_public_bytes),
            &tee_config,
        )
        .unwrap();

        // Verify decryption worked correctly
        assert_eq!(
            decrypted_input.tribute_draft_id,
            tribute_input.tribute_draft_id
        );
        assert_eq!(decrypted_input.cu_hashes, tribute_input.cu_hashes);
        assert_eq!(decrypted_input.worldwide_day, tribute_input.worldwide_day);
        assert_eq!(
            decrypted_input.settlement_currency,
            tribute_input.settlement_currency
        );
        assert_eq!(
            decrypted_input.settlement_base_amount,
            tribute_input.settlement_base_amount
        );
        assert_eq!(decrypted_input.owner, tribute_input.owner);
    }

    #[test]
    fn test_instantiate_with_valid_tee_config() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let owner_addr = deps.api.addr_make("owner");
        let info = MessageInfo {
            sender: owner_addr.clone(),
            funds: vec![],
        };

        // Generate valid keypair
        let (private_key, _) = generate_keypair();
        let salt = [1u8; 32];

        let tee_setup = TeeSetup {
            private_key: Base58Binary::from(private_key),
            salt: Base58Binary::from(salt),
        };

        let instantiate_msg = InstantiateMsg {
            owner: Some(info.sender.clone()),
            tribute_address: None,
            tee_config: Some(tee_setup),
            zk_config: None,
        };

        let result = instantiate(deps.as_mut(), env, info, instantiate_msg);
        assert!(result.is_ok());
    }

    #[test]
    fn test_instantiate_with_invalid_private_key_length() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let owner_addr = deps.api.addr_make("owner");
        let info = MessageInfo {
            sender: owner_addr.clone(),
            funds: vec![],
        };

        let tee_setup = TeeSetup {
            private_key: Base58Binary::from([1u8; 16]), // Invalid length
            salt: Base58Binary::from([1u8; 32]),
        };

        let instantiate_msg = InstantiateMsg {
            owner: Some(info.sender.clone()),
            tribute_address: None,
            tee_config: Some(tee_setup),
            zk_config: None,
        };

        let result = instantiate(deps.as_mut(), env, info, instantiate_msg);
        assert!(matches!(result, Err(ContractError::InvalidKey {})));
    }

    #[test]
    fn test_instantiate_with_invalid_salt_length() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let owner_addr = deps.api.addr_make("owner");
        let info = MessageInfo {
            sender: owner_addr.clone(),
            funds: vec![],
        };

        let (private_key, _) = generate_keypair();

        let tee_setup = TeeSetup {
            private_key: Base58Binary::from(private_key),
            salt: Base58Binary::from([1u8; 16]), // Invalid length
        };

        let instantiate_msg = InstantiateMsg {
            owner: Some(info.sender.clone()),
            tribute_address: None,
            tee_config: Some(tee_setup),
            zk_config: None,
        };

        let result = instantiate(deps.as_mut(), env, info, instantiate_msg);
        assert!(matches!(result, Err(ContractError::InvalidSalt {})));
    }

    #[test]
    fn test_instantiate_without_tee_config() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let owner_addr = deps.api.addr_make("owner");
        let info = MessageInfo {
            sender: owner_addr.clone(),
            funds: vec![],
        };

        let instantiate_msg = InstantiateMsg {
            owner: Some(info.sender.clone()),
            tribute_address: None,
            tee_config: None, // No TEE config should be fine
            zk_config: None,
        };

        let result = instantiate(deps.as_mut(), env, info, instantiate_msg);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_tee_config() {
        // Generate valid keypair
        let (private_key, _) = generate_keypair();
        let salt = [1u8; 32];

        // Test valid config
        let valid_config = TeeSetup {
            private_key: Base58Binary::from(private_key),
            salt: Base58Binary::from(salt),
        };
        assert!(validate_tee_config(&valid_config).is_ok());

        // Test invalid private key length
        let invalid_private_key = TeeSetup {
            private_key: Base58Binary::from([1u8; 16]), // Invalid length
            salt: Base58Binary::from(salt),
        };
        assert!(matches!(
            validate_tee_config(&invalid_private_key),
            Err(ContractError::InvalidKey {})
        ));

        // Test invalid salt length
        let invalid_salt = TeeSetup {
            private_key: Base58Binary::from(private_key),
            salt: Base58Binary::from([1u8; 16]), // Invalid length
        };
        assert!(matches!(
            validate_tee_config(&invalid_salt),
            Err(ContractError::InvalidSalt {})
        ));
    }
}
