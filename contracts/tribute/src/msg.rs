use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Uint128};
use outbe_nft::msg::{CollectionInfoMsg, Cw721InstantiateMsg};
use outbe_utils::denom::Denom;

#[cw_serde]
pub struct TributeCollectionExtension {
    pub symbolic_rate: Decimal,
    pub native_token: Denom,
    pub price_oracle: Addr,
}

pub type InstantiateMsg = Cw721InstantiateMsg<TributeCollectionExtension>;

#[cw_serde]
pub enum ExecuteMsg {
    UpdateMinterOwnership(cw_ownable::Action),
    UpdateCreatorOwnership(cw_ownable::Action),
    UpdateBurnerOwnership(cw_ownable::Action),

    /// The creator is the only one eligible to update `CollectionInfo`.
    UpdateCollectionInfo {
        collection_info: CollectionInfoMsg<Option<TributeCollectionExtension>>,
    },

    /// Mint a new NFT, can only be called by the contract minter
    Mint {
        /// Unique ID of the NFT
        token_id: String,
        /// The owner of the newly minter NFT
        owner: String,
        /// Universal resource identifier for this NFT
        /// Should point to a JSON file that conforms to the ERC721
        /// Metadata JSON Schema
        token_uri: Option<String>,
        /// Any custom extension used by this contract
        extension: Box<MintExtension>,
    },

    /// Burn an NFT the sender has access to
    Burn {
        token_id: String,
    },
    /// Removes all tributes previously submitted
    BurnAll {},
}

#[cw_serde]
pub struct MintExtension {
    pub data: TributeMintData,
}

#[cw_serde]
pub struct TributeMintData {
    pub tribute_id: String,
    /// Date of the Tribute creation
    pub worldwide_day: u64,
    pub owner: String,
    /// Value of the Tribute in Settlement Tokens
    pub settlement_amount_minor: Uint128,
    /// Tribute settlement token
    pub settlement_currency: Denom,
    /// Value of the Tribute in Native Coins
    pub nominal_qty_minor: Uint128,
    /// Price in Native coins with a rate on the moment of the transaction
    pub tribute_price_minor: Decimal,
}

#[cw_serde]
pub enum MigrateMsg {
    Migrate {},
}
