use crate::types::State;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Timestamp, Uint128};
use outbe_nft::msg::Cw721InstantiateMsg;
use outbe_utils::date::WorldwideDay;
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
    /// Update tokens with floor_price less than threshold to Qualified status
    PriceUpdate {
        /// Price threshold - tokens with floor_price < threshold will be updated
        price_threshold: Decimal,
    },
    /// Update the address that can call PriceUpdate
    UpdatePriceUpdater {
        /// Address of the new price updater
        price_updater: Option<String>,
    },
    #[cfg(feature = "demo")]
    BurnAll { batch_size: Option<usize> },
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
    /// Worldwide day of the tribute in YYYYMMDD format
    pub worldwide_day: WorldwideDay,
    pub settlement_currency: Denom,
    pub symbolic_rate: Decimal,
    pub floor_rate: Decimal,
    pub nominal_price: Decimal,
    pub issuance_price: Decimal,
    pub gratis_load_minor: Uint128,
    pub floor_price: Decimal,
    pub state: State,
    pub owner: String,
    pub qualified_at: Option<Timestamp>,
    pub is_touch: bool,
}

#[cw_serde]
pub enum MigrateMsg {
    Migrate {},
}
