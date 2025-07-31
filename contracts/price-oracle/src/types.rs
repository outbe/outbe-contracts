use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Timestamp};
use outbe_utils::denom::Denom;

// Keep legacy types for compatibility
#[cw_serde]
pub struct TokenPairPrice {
    pub token1: Denom,
    pub token2: Denom,
    pub day_type: DayType,
    pub price: Decimal,
}

#[cw_serde]
pub enum DayType {
    Green,
    Red,
}

// New types
#[cw_serde]
pub struct PriceData {
    pub price: Decimal,
    pub timestamp: Timestamp,
    pub open: Option<Decimal>,
    pub high: Option<Decimal>,
    pub low: Option<Decimal>,
    pub close: Option<Decimal>,
}

#[cw_serde]
pub struct TokenPair {
    pub token1: Denom,
    pub token2: Denom,
}

#[cw_serde]
pub struct UpdatePriceParams {
    pub token1: Denom,
    pub token2: Denom,
    pub price: Decimal,
    pub open: Option<Decimal>,
    pub high: Option<Decimal>,
    pub low: Option<Decimal>,
    pub close: Option<Decimal>,
}
