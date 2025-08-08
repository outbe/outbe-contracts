use crate::types::State;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Timestamp, Uint128};
use outbe_nft::msg::Cw721InstantiateMsg;
use outbe_utils::denom::Denom;

/// Custom collection extension for instantiate
#[cw_serde]
pub struct NodCollectionExtension {}

pub type InstantiateMsg = Cw721InstantiateMsg<NodCollectionExtension>;

/// Execute messages for Nod contract
#[cw_serde]
pub enum ExecuteMsg {
    /// Submit (mint) a new Nod NFT
    Submit {
        /// Unique ID of the NFT
        token_id: String,
        /// Owner of the newly minted NFT
        owner: String,
        /// Custom extension data for the Nod
        extension: Box<SubmitExtension>,
    },
    /// Burn an existing Nod NFT
    Burn { token_id: String },
    #[cfg(feature = "demo")]
    BurnAll {},
}

/// Extension data for submit (mint)
#[cw_serde]
pub struct SubmitExtension {
    pub entity: NodEntity,
    pub created_at: Option<Timestamp>,
}

/// Entity data for each Nod NFT
#[cw_serde]
pub struct NodEntity {
    pub nod_id: String,
    pub settlement_currency: Denom,
    pub symbolic_rate: Decimal,
    pub floor_rate: Uint128,
    pub nominal_price_minor: Decimal,
    pub issuance_price_minor: Decimal,
    pub gratis_load_minor: Uint128,
    pub floor_price_minor: Decimal,
    pub state: State,
    pub owner: String,
    pub qualified_at: Option<Timestamp>,
}

/// Migrate message for Nod contract
#[cw_serde]
pub enum MigrateMsg {}
