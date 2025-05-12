use crate::error::ContractError;
use crate::msg::{
    ConsumptionUnitEntity, ConsumptionUnitExtensionUpdate, ExecuteMsg, InstantiateMsg, MigrateMsg,
    MintExtension,
};
use crate::state::HASHES;
use crate::types::{CUConfig, ConsumptionUnitData, ConsumptionUnitNft, ConsumptionUnitState};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Api, Decimal, DepsMut, Env, Event, MessageInfo, Response, Uint128,
};
use cw_ownable::OwnershipError;
use outbe_nft::error::Cw721ContractError;
use outbe_nft::execute::assert_minter;
use outbe_nft::state::{CollectionInfo, Cw721Config};
use sha2::{Digest, Sha256};

const CONTRACT_NAME: &str = "gemlabs.io:consumption-unit";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let cfg = CUConfig {
        settlement_token: msg.collection_info_extension.settlement_token.clone(),
        native_token: msg.collection_info_extension.native_token.clone(),
        price_oracle: msg.collection_info_extension.price_oracle.clone(),
    };

    let collection_info = CollectionInfo {
        name: msg.name,
        symbol: msg.symbol,
        updated_at: env.block.time,
    };

    let config = Cw721Config::<ConsumptionUnitData, CUConfig>::default();
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
        .add_attribute("action", "consumption-unit::instantiate")
        .add_event(
            Event::new("consumption-unit::instantiate")
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
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    token_id: String,
    update: ConsumptionUnitExtensionUpdate,
) -> Result<Response, ContractError> {
    let config = Cw721Config::<ConsumptionUnitData, CUConfig>::default();

    match update {
        ConsumptionUnitExtensionUpdate::UpdateVector { new_vector_id } => {
            let mut current_nft_info = config.nft_info.load(deps.storage, &token_id)?;
            if current_nft_info.owner != info.sender {
                return Err(ContractError::Cw721ContractError(
                    Cw721ContractError::Ownership(OwnershipError::NotOwner),
                ));
            }

            if current_nft_info.extension.state == ConsumptionUnitState::Selected {
                return Err(ContractError::WrongInput {});
            }

            verify_vector(new_vector_id)?;

            current_nft_info.extension =
                current_nft_info.extension.update_vector(new_vector_id, env);

            config
                .nft_info
                .save(deps.storage, &token_id, &current_nft_info)?;

            Ok(Response::new()
                .add_attribute("action", "consumption-unit::update_nft_info")
                .add_event(
                    Event::new("consumption-unit::update_nft_info")
                        .add_attribute("token_id", token_id)
                        .add_attribute("new_commitment_pool_id", new_vector_id.to_string()),
                ))
        }
    }
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

    verify_vector(extension.vector)?;

    if entity.hashes.is_empty()
        || entity.nominal_quantity == Uint128::zero()
        || entity.consumption_value == Uint128::zero()
    {
        return Err(ContractError::WrongInput {});
    }

    verify_signature(
        deps.api,
        entity.clone(),
        extension.signature,
        extension.public_key,
    )?;

    let config = Cw721Config::<ConsumptionUnitData, CUConfig>::default();

    // create the token
    let data = ConsumptionUnitData {
        consumption_value: entity.consumption_value,
        nominal_quantity: entity.nominal_quantity,
        nominal_currency: entity.nominal_currency,
        vector: extension.vector,
        state: ConsumptionUnitState::Reflected,
        floor_price: Decimal::one(), // TODO query from Oracle
        hashes: entity.hashes.clone(),
        created_at: env.block.time,
        updated_at: env.block.time,
    };

    let token = ConsumptionUnitNft {
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
        HASHES.update(deps.storage, &hash, |old| match old {
            Some(_) => Err(ContractError::HashAlreadyExists {}),
            None => Ok(token_id.clone()),
        })?;
    }

    config.increment_tokens(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "consumption-unit::mint")
        .add_event(
            Event::new("consumption-unit::mint")
                .add_attribute("token_id", token_id)
                .add_attribute("owner", owner),
        ))
}

fn verify_signature(
    api: &dyn Api,
    entity: ConsumptionUnitEntity,
    signature: String,
    public_key: String,
) -> Result<(), ContractError> {
    let signature_bytes = match hex::decode(signature) {
        Ok(data) => data,
        Err(_) => return Err(ContractError::WrongInput {}),
    };
    let public_key_bytes = match hex::decode(public_key) {
        Ok(data) => data,
        Err(_) => return Err(ContractError::WrongInput {}),
    };

    let serialized_entity = to_json_binary(&entity)?;
    let data_hash = Sha256::digest(serialized_entity.clone());

    let signature_ok = api.secp256k1_verify(&data_hash, &signature_bytes, &public_key_bytes)?;
    if signature_ok {
        Ok(())
    } else {
        Err(ContractError::WrongDigest {})
    }
}

/// Verifies that the given vector id is correct.
/// NB: should be in sync with Vector smart contract.
/// NB: we do not store ref to that contract to save gas
fn verify_vector(new_vector_id: u16) -> Result<(), ContractError> {
    if (1..=16).contains(&new_vector_id) {
        return Ok(());
    }
    Err(ContractError::WrongVector {})
}

fn execute_burn(
    deps: DepsMut,
    _env: &Env,
    info: &MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    let config = Cw721Config::<ConsumptionUnitData, CUConfig>::default();
    // TODO verify ownership
    // let token = config.nft_info.load(deps.storage, &token_id)?;
    // check_can_send(deps.as_ref(), env, info.sender.as_str(), &token)?;

    config.nft_info.remove(deps.storage, &token_id)?;
    config.decrement_tokens(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "consumption-unit::burn")
        .add_event(
            Event::new("consumption-unit::burn")
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
    use crate::contract::verify_signature;
    use crate::msg::ConsumptionUnitEntity;
    use cosmwasm_schema::schemars::_serde_json::from_str;
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::to_json_binary;
    use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
    use sha2::{Digest, Sha256};
    use std::str::FromStr;

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
            "consumption_value": "100",
            "nominal_quantity": "100",
            "nominal_currency": "usd",
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
        let signature_hex = hex::encode(sig.serialize_compact());
        println!("Compact Signature (64 bytes): {}", signature_hex);

        // Serialize public key compressed (33 bytes)
        let public_key_hex = hex::encode(public_key.serialize());

        println!("Public key (compressed, 33 bytes): {}", public_key_hex);

        assert_eq!(
            verify_signature(&deps.api, entity, signature_hex, public_key_hex),
            Ok(())
        );
    }
}
