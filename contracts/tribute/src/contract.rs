use crate::error::ContractError;
use crate::msg::{
    ConsumptionUnitEntity, ConsumptionUnitExtensionUpdate, ExecuteMsg, InstantiateMsg, MigrateMsg,
    MintExtension,
};
use crate::state::HASHES;
use crate::types::{Status, TributeConfig, TributeData, TributeNft};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Api, Decimal, DepsMut, Env, Event, HexBinary, MessageInfo, Response, Uint128,
};
use outbe_nft::error::Cw721ContractError;
use outbe_nft::execute::assert_minter;
use outbe_nft::state::{CollectionInfo, Cw721Config};
use sha2::{Digest, Sha256};

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
        settlement_token: msg.collection_info_extension.settlement_token.clone(),
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
            extension,
        } => execute_mint(deps, &env, &info, token_id, owner, *extension),
        ExecuteMsg::Burn { token_id } => execute_burn(deps, &env, &info, token_id),
        ExecuteMsg::UpdateNftInfo {
            token_id,
            extension,
        } => execute_update_nft_info(deps, &env, &info, token_id, extension),
    }
}

fn execute_update_nft_info(
    _deps: DepsMut,
    _env: &Env,
    _info: &MessageInfo,
    _token_id: String,
    _update: ConsumptionUnitExtensionUpdate,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}

#[allow(clippy::too_many_arguments)]
fn execute_mint(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    token_id: String,
    owner: String,
    extension: MintExtension,
) -> Result<Response, ContractError> {
    assert_minter(deps.storage, &info.sender)?;
    // validate owner
    let owner_addr = deps.api.addr_validate(&owner)?;

    let entity = extension.entity;
    if entity.token_id != token_id || entity.owner != owner {
        return Err(ContractError::WrongInput {});
    }

    if entity.hashes.is_empty() || entity.minor_value_settlement == Uint128::zero() {
        return Err(ContractError::WrongInput {});
    }

    verify_signature(
        deps.api,
        entity.clone(),
        extension.signature,
        extension.public_key,
    )?;

    let config = Cw721Config::<TributeData, TributeConfig>::default();
    let col_config = config.collection_config.load(deps.storage)?;
    let exchange_rate = Decimal::one(); // TODO query from Oracle

    let (nominal_qty, load) = calc_sybolics(
        entity.minor_value_settlement,
        exchange_rate,
        col_config.symbolic_rate,
    );

    // create the token
    let data = TributeData {
        minor_value_settlement: entity.minor_value_settlement,
        nominal_price: exchange_rate,
        nominal_minor_qty: nominal_qty,
        vector: 1, // TODO: hardcode tmp
        status: Status::Accepted,
        symbolic_load: load,
        hashes: entity.hashes.clone(),
        created_at: env.block.time,
        updated_at: env.block.time,
    };

    let token = TributeNft {
        owner: owner_addr,
        extension: data,
    };

    config
        .nft_info
        .update(deps.storage, &token_id, |old| match old {
            Some(_) => Err(Cw721ContractError::Claimed {}),
            None => Ok(token),
        })?;

    for hash in entity.hashes {
        HASHES.update(deps.storage, &hash.to_hex(), |old| match old {
            Some(_) => Err(ContractError::HashAlreadyExists {}),
            None => Ok(token_id.clone()),
        })?;
    }

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
    settlement_value: Uint128,
    exchange_rate: Decimal,
    symbolic_rate: Decimal,
) -> (Uint128, Uint128) {
    let settlement_value_dec = Decimal::from_atomics(settlement_value, 0).unwrap();
    let nominal_qty = settlement_value_dec * exchange_rate;

    let symbolic_divisor = settlement_value_dec / nominal_qty * (Decimal::one() + symbolic_rate);
    let load = settlement_value_dec * symbolic_rate / symbolic_divisor;

    (nominal_qty.to_uint_floor(), load.to_uint_floor())
}

fn verify_signature(
    api: &dyn Api,
    entity: ConsumptionUnitEntity,
    signature: HexBinary,
    public_key: HexBinary,
) -> Result<(), ContractError> {
    let serialized_entity = to_json_binary(&entity)?;
    let data_hash = Sha256::digest(serialized_entity.clone());

    let signature_ok =
        api.secp256k1_verify(&data_hash, signature.as_slice(), public_key.as_slice())?;
    if signature_ok {
        Ok(())
    } else {
        Err(ContractError::WrongDigest {})
    }
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    match msg {
        MigrateMsg::Migrate {} => Ok(Response::new()),
    }
}

#[cfg(test)]
mod tests {
    use crate::contract::{calc_sybolics, verify_signature};
    use crate::msg::ConsumptionUnitEntity;
    use cosmwasm_schema::schemars::_serde_json::from_str;
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::{to_json_binary, Decimal, HexBinary, Uint128};
    use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
    use sha2::{Digest, Sha256};
    use std::str::FromStr;

    #[test]
    fn test_symbolics_calc() {
        let (nominal, load) = calc_sybolics(
            Uint128::new(1500u128),
            Decimal::from_str("0.2").unwrap(),
            Decimal::from_str("0.08").unwrap(),
        );
        assert_eq!(nominal, Uint128::new(300u128));
        assert_eq!(load, Uint128::new(22u128));
    }

    #[test]
    fn test_signature_creation() {
        let deps = mock_dependencies();
        let secp = Secp256k1::new();

        // prepare test keys
        let private_key =
            SecretKey::from_str("4236627b5a03b3f2e601141a883ccdb23aeef15c910a0789e4343aad394cbf6d")
                .unwrap();
        let public_key = PublicKey::from_secret_key(&secp, &private_key);

        // prepare raw json data
        let raw_json = r#"{
            "token_id": "1",
            "owner": "cosmwasm1j2mmggve9m6fpuahtzvwcrj3rud9cqjz9qva39cekgpk9vprae8s4haddx",
            "minor_value_settlement": "100",
            "hashes": [
              "872be89dd82bcc6cf949d718f9274a624c927cfc91905f2bbb72fa44c9ea876d"
            ]
        }"#;
        let entity: ConsumptionUnitEntity = from_str(raw_json).unwrap();

        // sign the data
        let message_binary = to_json_binary(&entity).unwrap();
        println!("message_binary {:?}", hex::encode(message_binary.clone()));
        let message_hash = Sha256::digest(message_binary);

        println!("message_hash {:?}", hex::encode(message_hash));

        // Sign the hashed message
        let msg = Message::from_digest_slice(&message_hash).unwrap();
        // Sign message (produces low-S normalized signature)
        let sig = secp.sign_ecdsa(&msg, &private_key);

        // verify the signature using standard lib
        secp.verify_ecdsa(&msg, &sig, &public_key)
            .map(|_| println!("Signature is valid."))
            .unwrap();

        // Verify signature on the smart contract side
        // Serialize signature in compact 64-byte form
        let signature_hex = HexBinary::from(sig.serialize_compact());
        println!("Compact Signature (64 bytes): {}", signature_hex);

        // Serialize public key compressed (33 bytes)
        let public_key_hex = HexBinary::from(public_key.serialize());

        println!("Public key (compressed, 33 bytes): {}", public_key_hex);

        assert_eq!(
            verify_signature(&deps.api, entity, signature_hex, public_key_hex),
            Ok(())
        );
    }
}
