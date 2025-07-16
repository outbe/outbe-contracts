use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Timestamp, Uint128};
use outbe_nft::state::NftInfo;
use outbe_nft::traits::{Cw721CollectionConfig, Cw721CustomMsg, Cw721State};
use outbe_utils::denom::Denom;

/// Configuration for the Nod NFT collection (empty)
#[cw_serde]
pub struct NodConfig {}

impl Cw721CollectionConfig for NodConfig {}

/// Public data for each Nod NFT
#[cw_serde]
pub struct NodData {
    /// Identifier of the Nod
    pub nod_id: String,
    /// Settlement Currency
    pub settlement_currency: Denom,
    /// Symbolic rate
    pub symbolic_rate: Decimal,
    /// Account specific, from Lysis
    pub floor_rate: Uint128,
    /// From Tribute
    pub nominal_price_minor: Uint128,
    /// coen Price at the moment of Nod issuance
    pub issuance_price_minor: Uint128,
    /// From Tribute Symbolic Load
    pub gratis_load_minor: Uint128,
    /// Floor price in minor units
    pub floor_price_minor: Uint128,
    /// Current state of the Nod
    pub state: State,
    /// Address entitled to mine Gratis
    pub owner: String,
    /// Creation timestamp
    pub issued_at: Timestamp,
    /// Timestamp when the Nod was qualified
    pub qualified_at: Option<Timestamp>,
}

/// Possible states for a Nod
#[cw_serde]
pub enum State {
    Issued,
    Qualified,
}

pub type NodNft = NftInfo<NodData>;

impl Cw721State for NodData {}
impl Cw721CustomMsg for NodData {}
