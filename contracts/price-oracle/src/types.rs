use cosmwasm_schema::cw_serde;
use cosmwasm_std::Decimal;
use outbe_utils::denom::Denom;

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
