use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Uint128, Uint64};
use outbe_utils::date::Iso8601Date;
use outbe_utils::denom::CurrencyCode;
use outbe_utils::Base58Binary;

#[cw_serde]
pub struct TributeInputPayload {
    /// ID of the draft tribute
    pub tribute_draft_id: Base58Binary,
    /// Owner is a derivative address on L2 network based on blake3 hashing
    pub owner: Base58Binary,
    /// ISO 8601
    pub worldwide_day: Iso8601Date,
    /// ISO 4217
    pub settlement_currency: CurrencyCode,
    /// Amount expressed in natural units, `settlement_base_amount >= 0`
    pub settlement_base_amount: Uint64,
    /// Amount expressed in fractional units, `0 >= settlement_atto_amount < 1e18`
    pub settlement_atto_amount: Uint128,
    /// Quantity expressed in natural units, `nominal_base_amount >= 0`
    pub nominal_base_amount: Uint64,
    /// Amount expressed in fractional units, `0 >= nominal_atto_amount < 1e18`
    pub nominal_atto_amount: Uint128,
    pub cu_hashes: Vec<Base58Binary>,
}
