use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, HexBinary, Timestamp, Uint128};
use cw20::Denom;
use outbe_nft::msg::Cw721InstantiateMsg;

#[cw_serde]
pub struct ConsumptionUnitCollectionExtension {
    pub settlement_token: Denom,
    pub symbolic_rate: Decimal,
    pub native_token: Denom,
    /// Address of the price Oracle to query floor prices
    pub price_oracle: Addr,
}

pub type InstantiateMsg = Cw721InstantiateMsg<ConsumptionUnitCollectionExtension>;

#[cw_serde]
pub enum ExecuteMsg {
    /// Mint a new NFT, can only be called by the contract minter
    Mint {
        /// Unique ID of the NFT
        token_id: String,
        /// The owner of the newly minter NFT
        owner: String,
        /// Any custom extension used by this contract
        extension: Box<MintExtension>,
    },

    /// Burn an NFT the sender has access to
    Burn {
        token_id: String,
    },
    // todo remove after demo
    BurnAll {},

    /// Extension msg
    UpdateNftInfo {
        token_id: String,
        extension: TributeExtensionUpdate,
    },
}

#[cw_serde]
pub struct MintExtension {
    pub entity: TributeEntity,
    /// Serialized "compact" signature (64 bytes) of the `entity` in hex
    pub signature: HexBinary,
    /// Serialized according to SEC 2 (33 or 65 bytes) public key in hex
    pub public_key: HexBinary,
    /// Time of the Tribute NFT for demo
    pub tribute_date: Option<Timestamp>,
}

#[cw_serde]
pub struct TributeEntity {
    pub token_id: String,
    pub owner: String,
    /// Value of the Tribute in Settlement Tokens
    pub minor_value_settlement: Uint128,
    /// Hashes identifying consumption records batch. Each hash should be a valid unique
    /// sha256 hash in hex format
    pub hashes: Vec<HexBinary>,
}

#[cw_serde]
pub enum TributeExtensionUpdate {}

#[cw_serde]
pub enum MigrateMsg {
    Migrate {},
}
