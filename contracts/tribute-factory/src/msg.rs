use crate::types::TributeInputPayload;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, HexBinary};

pub type TributeMsg = tribute::msg::ExecuteMsg;
pub type TributeMintExtension = tribute::msg::MintExtension;
pub type TributeMintData = tribute::msg::TributeMintData;

#[cw_serde]
pub struct InstantiateMsg {
    /// Tribute smart contract address
    pub tribute_address: Option<Addr>,
    /// Sets the owner.
    pub owner: Option<Addr>,
    /// Trusted execution environment config
    pub tee_config: Option<TeeSetup>,
    pub zk_config: Option<ZkSetup>,
}

#[cw_serde]
pub struct TeeSetup {
    /// Ed25519 private key for messages encryption
    pub private_key: HexBinary,
    /// Salt to be used in hashing operations
    pub salt: HexBinary,
}

#[cw_serde]
pub struct ZkSetup {
    /// ZK circuit used to verify proofs
    pub circuit: HexBinary,
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
        /// Zero knowledge proof
        zk_proof: ZkProof,
    },

    /// Accepts raw tribute data and mints a new Tribute
    /// TEST PURPOSE ONLY
    OfferInsecure {
        tribute_input: TributeInputPayload,
        zk_proof: ZkProof,
        tribute_owner_l1: Option<Addr>,
    },
    BurnAll {},
}

#[cw_serde]
pub struct ZkProof {
    /// Zero knowledge proof as Structured Reference String and based on PlonK algorithm
    pub proof: HexBinary,
    /// ZK public data
    pub public_data: ZkProofPublicData,
    /// ZK verification key
    pub verification_key: HexBinary,
}

#[cw_serde]
pub struct ZkProofPublicData {
    /// Public key of the user that created a proof
    pub public_key: HexBinary,
    /// Merkle root of the L2 state
    pub merkle_root: HexBinary,
}
