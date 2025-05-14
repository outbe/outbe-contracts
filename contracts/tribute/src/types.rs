use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Env, Timestamp, Uint128};
use cw20::Denom;
use outbe_nft::state::NftInfo;
use outbe_nft::traits::Cw721CollectionConfig;

/// ConsumptionUnit contract config
#[cw_serde]
pub struct TributeConfig {
    pub settlement_token: Denom,
    pub symbolic_rate: Decimal,
    pub native_token: Denom,
    pub price_oracle: Addr,
}

impl Cw721CollectionConfig for TributeConfig {}

/// ConsumptionUnit public data
#[cw_serde]
pub struct TributeData {
    /// Value of the Tribute in Settlement Tokens
    pub minor_value_settlement: Uint128,
    /// Value of the Tribute in Native Coins
    pub nominal_minor_qty: Uint128,
    /// Price in Native coins with a rate on the moment of the transaction
    pub nominal_price: Decimal,
    /// Identifier for the Tier to which the user allocated the Tribute.
    /// Can be set/updated by the user before raffle selection
    pub vector: u16,
    /// Signals an eligible interest to the network Gratis qty
    pub symbolic_load: Uint128,
    /// State of the record
    pub status: Status,
    /// Hashes identifying consumption records batch. Each hash should be a valid unique
    /// sha256 hash in hex format
    pub hashes: Vec<String>,
    /// Time when the Tribute NFT was created on the network
    pub created_at: Timestamp,
    /// Last updated time
    pub updated_at: Timestamp,
}

#[cw_serde]
pub enum Status {
    /// Accepted on the Network (Vector and floorPrice can be changed)
    Accepted,
    /// Submitted on a Red Day; not eligible for Raffle
    Muted,
    /// Participating in Raffle
    /// (Commitment pool and consequently floorPrice can be changed)
    Raffle,
    /// Was selected in Raffle
    /// (Vector and floorPrice can’t be changed)
    Recognized,
    /// all Raffles completed and tribute didn't receive recognition
    Voided,
}

pub type TributeNft = NftInfo<TributeData>;

impl outbe_nft::traits::Cw721State for TributeData {}
impl outbe_nft::traits::Cw721CustomMsg for TributeData {}

impl TributeData {
    pub fn update_vector(mut self, new_vector_id: u16, env: &Env) -> Self {
        self.vector = new_vector_id;
        self.updated_at = env.block.time;
        self
    }
}
