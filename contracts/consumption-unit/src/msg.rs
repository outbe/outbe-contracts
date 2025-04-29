use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw20::Denom;
use q_nft::msg::Cw721InstantiateMsg;

#[cw_serde]
pub struct ConsumptionUnitCollectionExtension {
    pub settlement_token: Denom,
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
        extension: MintExtension,
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
    /// Sha256 hash of the given `entity` in hex format
    pub digest: String,
}

#[cw_serde]
pub struct ConsumptionUnitEntity {
    pub token_id: String,
    pub owner: String,
    /// The value of Consumption Unit in Settlement Tokens
    pub consumption_value: Uint128,
    /// Sum of Nominal Qty from Consumption Records
    pub nominal_quantity: Uint128,
    /// Nominal currency from Consumption Records
    pub nominal_currency: String,
    /// Where the CU is allocated by the User.
    /// A user can change commitment Pool at any time prior to CU NFT selection in raffle
    pub commitment_tier: u16,
    /// Hashes identifying consumption records batch. Each hash should be a valid unique
    /// sha256 hash in hex format
    pub hashes: Vec<String>,
}

#[cw_serde]
pub enum ConsumptionUnitExtensionUpdate {
    /// Updates the pool id for the given NFT, can be performed by user only.
    /// When updating the pool a new price will be fetched.
    UpdatePool { new_commitment_tier_id: u16 },
}

#[cw_serde]
pub enum MigrateMsg {
    Migrate {},
}
