use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, HexBinary, Timestamp, Uint128};
use cw20::Denom;
use outbe_nft::msg::{CollectionInfoMsg, Cw721InstantiateMsg};

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
    /// Serialized "compact" signature (64 bytes) of the `entity` in hex
    pub signature: HexBinary,
    /// Serialized according to SEC 2 (33 or 65 bytes) public key in hex
    pub public_key: HexBinary,
}

#[cw_serde]
pub struct TributeMintData {
    pub token_id: String,
    pub owner: String,
    /// Value of the Tribute in Settlement Tokens
    pub settlement_value: Uint128,
    /// Tribute settlement token
    pub settlement_token: Denom,
    /// Date of the Tribute creation
    pub tribute_date: Option<Timestamp>,
    /// Hashes identifying consumption records batch. Each hash should be a valid unique
    /// sha256 hash in hex format
    pub hashes: Vec<HexBinary>,
}

#[cw_serde]
pub enum MigrateMsg {
    Migrate {},
}
