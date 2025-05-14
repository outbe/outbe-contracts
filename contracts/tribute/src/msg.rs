use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Uint128};
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
    Burn { token_id: String },

    /// Extension msg
    UpdateNftInfo {
        token_id: String,
        extension: ConsumptionUnitExtensionUpdate,
    },
}

#[cw_serde]
pub struct MintExtension {
    pub entity: ConsumptionUnitEntity,
    /// Where the CU is allocated by the User.
    /// A user can change Vector at any time prior to CU NFT selection in raffle
    pub vector: u16,
    /// Serialized "compact" signature (64 bytes) of the `entity` in hex
    pub signature: String,
    /// Serialized according to SEC 2 (33 or 65 bytes) public key in hex
    pub public_key: String,
}

#[cw_serde]
pub struct ConsumptionUnitEntity {
    pub token_id: String,
    pub owner: String,
    /// Value of the Tribute in Settlement Tokens
    pub minor_value_settlement: Uint128,
    /// Hashes identifying consumption records batch. Each hash should be a valid unique
    /// sha256 hash in hex format
    pub hashes: Vec<String>,
}

#[cw_serde]
pub enum ConsumptionUnitExtensionUpdate {
    /// Updates the vector id for the given NFT, can be performed by user only.
    /// When updating the vector, a new price will be fetched.
    UpdateVector { new_vector_id: u16 },
}

#[cw_serde]
pub enum MigrateMsg {
    Migrate {},
}
