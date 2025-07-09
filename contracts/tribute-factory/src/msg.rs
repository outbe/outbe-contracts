use crate::types::TributeInputPayload;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, HexBinary};

#[cw_serde]
pub struct InstantiateMsg {
    /// Tribute smart contract address
    pub tribute_address: Option<Addr>,
    /// Sets the owner.
    pub owner: Option<Addr>,
    /// Trusted execution environment config
    pub tee_config: Option<TeeSetup>,
}

#[cw_serde]
pub struct TeeSetup {
    /// Ed25519 private key for messages encryption
    pub private_key: HexBinary,
    /// Salt to be used in hashing operations
    pub salt: HexBinary,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        new_owner: Option<Addr>,
        new_tribute_address: Option<Addr>,
        new_tee_config: Option<TeeSetup>,
    },

    /// Accepts encrypted data and mints a new Tribute
    Offer {
        /// Encrypted TributeInputPayload
        cipher_text: HexBinary,
        /// Public nonce to decrypt the data
        nonce: HexBinary,
        /// Ephemeral public key to decrypt the data
        ephemeral_pubkey: HexBinary,
        // TODO: TBD zk proofs
        // zk_proof:
    },

    /// Accepts raw tribute data and mints a new Tribute
    /// TEST PURPOSE ONLY
    OfferInsecure {
        tribute_input: TributeInputPayload,
        // TODO: TBD zk proofs
        // zk_proof:
    },
}
