use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Timestamp, Uint128};
use cw20::Denom;
use outbe_nft::state::NftInfo;
use outbe_nft::traits::Cw721CollectionConfig;

/// ConsumptionUnit contract config
#[cw_serde]
pub struct TributeConfig {
    pub symbolic_rate: Decimal,
    pub native_token: Denom,
    pub price_oracle: Addr,
}

impl Cw721CollectionConfig for TributeConfig {}

/// ConsumptionUnit public data
#[cw_serde]
pub struct TributeData {
    /// Value of the Tribute in Settlement Tokens
    pub settlement_amount: Uint128,
    /// Tribute settlement token
    pub settlement_currency: Denom,
    /// Value of the Tribute in Native Coins
    pub nominal_qty: Uint128,
    /// Price in Native coins with a rate on the moment of the transaction
    pub tribute_rate: Decimal,
    /// Signals an eligible interest to the network
    pub symbolic_load: Uint128,
    /// Date of Consumption
    pub worldwide_day: Timestamp,

    pub fidelity_index: i32,
    /// Time when the Tribute NFT was created on the network
    pub created_at: Timestamp,
}

pub type TributeNft = NftInfo<TributeData>;

impl outbe_nft::traits::Cw721State for TributeData {}
impl outbe_nft::traits::Cw721CustomMsg for TributeData {}
