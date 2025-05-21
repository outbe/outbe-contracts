use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Timestamp, Uint128};
use cw20::Denom;
use outbe_nft::state::NftInfo;
use outbe_nft::traits::{Cw721CollectionConfig, Cw721CustomMsg, Cw721State};

/// Configuration for the Nod NFT collection (empty)
#[cw_serde]
pub struct NodConfig {}

impl Cw721CollectionConfig for NodConfig {}

/// Public data for each Nod NFT
#[cw_serde]
pub struct NodData {
    /// Identifier of the Nod
    pub nod_id: String,
    /// Settlement token denomination
    pub settlement_token: Denom,
    /// Symbolic rate
    pub symbolic_rate: Decimal,
    /// Nominal minor rate at transaction time
    pub nominal_minor_rate: Uint128,
    /// Issuance minor rate at issuance time
    pub issuance_minor_rate: Uint128,
    /// Symbolic minor load
    pub symbolic_minor_load: Uint128,
    /// Vector minor rate from account
    pub vector_minor_rate: Uint128,
    /// Floor minor price threshold
    pub floor_minor_price: Uint128,
    /// Current state of the Nod
    pub state: State,
    /// Address entitled to claim the Nod
    pub address: String,
    /// Creation timestamp
    pub created_at: Timestamp,
}

/// Possible states for a Nod
#[cw_serde]
pub enum State {
    Issued,
    Settled,
}

pub type NodNft = NftInfo<NodData>;

impl Cw721State for NodData {}
impl Cw721CustomMsg for NodData {}
