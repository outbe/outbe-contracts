use crate::error::ContractError;
use crate::state::TeeConfig;
use crate::types::TributeInputPayload;
use chacha20poly1305::aead::Aead;
use chacha20poly1305::{ChaCha20Poly1305, KeyInit, Nonce};
use cosmwasm_std::{Uint128, Uint64};
use curve25519_dalek::{MontgomeryPoint, Scalar};
use outbe_utils::Base58Binary;

fn generate_keypair() -> ([u8; 32], [u8; 32]) {
    use rand::rngs::OsRng;
    use rand::RngCore;

    let mut private_key_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut private_key_bytes);
    let private_key_scalar = Scalar::from_bytes_mod_order(private_key_bytes);

    let public_key_point = curve25519_dalek::constants::X25519_BASEPOINT * private_key_scalar;
    let public_key_bytes = public_key_point.to_bytes();

    (private_key_bytes, public_key_bytes)
}

fn encrypt_tribute_input(
    tribute_input: &TributeInputPayload,
    contract_public_key: &[u8; 32],
) -> Result<(Base58Binary, Base58Binary, Base58Binary), ContractError> {
    use rand::rngs::OsRng;
    use rand::RngCore;

    // Generate ephemeral keypair for a client
    let mut ephemeral_private_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut ephemeral_private_bytes);
    let ephemeral_private_scalar = Scalar::from_bytes_mod_order(ephemeral_private_bytes);
    let ephemeral_public_point =
        curve25519_dalek::constants::X25519_BASEPOINT * ephemeral_private_scalar;
    let ephemeral_public_bytes = ephemeral_public_point.to_bytes();

    // Perform ECDH
    let contract_public_point = MontgomeryPoint(*contract_public_key);
    let shared_secret = contract_public_point * ephemeral_private_scalar;

    // Serialize tribute input
    let plaintext = cosmwasm_std::to_json_binary(tribute_input)
        .map_err(|_| ContractError::InvalidPayload {})?
        .to_vec();

    // Generate random nonce
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);

    // Encrypt
    let cipher = ChaCha20Poly1305::new((&shared_secret.to_bytes()).into());
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|_| ContractError::DecryptionFailed {})?;

    Ok((
        Base58Binary::from(ciphertext),
        Base58Binary::from(nonce_bytes),
        Base58Binary::from(ephemeral_public_bytes),
    ))
}

#[test]
fn test_decrypt_tribute_input() {
    // Generate contract keypair
    let (private_key, public_key) = generate_keypair();

    // Create test tribute input
    let tribute_input = TributeInputPayload {
        tribute_draft_id: Base58Binary::from([42u8; 32]),
        cu_hashes: vec![Base58Binary::from([1u8; 32]), Base58Binary::from([2u8; 32])],
        worldwide_day: "2025-08-26".to_string(),
        settlement_currency: "usd".to_string(),
        settlement_base_amount: Uint64::new(1000),
        settlement_atto_amount: Uint128::zero(),
        nominal_base_qty: Uint64::new(500),
        nominal_atto_qty: Uint128::zero(),
        owner: Base58Binary::from("test_owner".as_bytes()),
    };

    // Encrypt tribute input (client side)
    let (cipher_text, nonce, ephemeral_pubkey) =
        encrypt_tribute_input(&tribute_input, &public_key).unwrap();

    // Test decryption by calling the contract function directly
    let tee_config = TeeConfig {
        private_key: Base58Binary::from(private_key),
        public_key: Base58Binary::from(public_key),
        salt: Base58Binary::from([1u8; 32]),
    };

    let decrypted_input = crate::contract::decrypt_tribute_input(
        &cipher_text,
        &nonce,
        &ephemeral_pubkey,
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
fn test_decrypt_tribute_input_invalid_key_size() {
    let tee_config = TeeConfig {
        private_key: Base58Binary::from([1u8; 16]), // Invalid size
        public_key: Base58Binary::from([1u8; 32]),
        salt: Base58Binary::from([1u8; 32]),
    };

    let result = crate::contract::decrypt_tribute_input(
        &Base58Binary::from([1u8; 32]),
        &Base58Binary::from([1u8; 12]),
        &Base58Binary::from([1u8; 32]),
        &tee_config,
    );

    assert!(matches!(result, Err(ContractError::InvalidKey {})));
}

#[test]
fn test_decrypt_tribute_input_invalid_nonce_size() {
    let tee_config = TeeConfig {
        private_key: Base58Binary::from([1u8; 32]),
        public_key: Base58Binary::from([1u8; 32]),
        salt: Base58Binary::from([1u8; 32]),
    };

    let result = crate::contract::decrypt_tribute_input(
        &Base58Binary::from([1u8; 32]),
        &Base58Binary::from([1u8; 8]), // Invalid size
        &Base58Binary::from([1u8; 32]),
        &tee_config,
    );

    assert!(matches!(result, Err(ContractError::InvalidNonce {})));
}
