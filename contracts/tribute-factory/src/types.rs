use cosmwasm_schema::cw_serde;
use cosmwasm_std::{HexBinary, Uint128, Uint64};
use outbe_utils::date::Iso8601Date;

#[cw_serde]
pub struct TributeInputPayload {
    /// ID of the draft tribute
    pub tribute_draft_id: HexBinary,
    /// Owner is a derivative address on L2 network based on blake3 hashing
    pub owner: String,
    /// ISO 8601
    pub worldwide_day: Iso8601Date,
    /// ISO 4217
    pub settlement_currency: String,
    /// Amount expressed in natural units, `settlement_base_amount >= 0`
    pub settlement_base_amount: Uint64,
    /// Amount expressed in fractional units, `0 >= settlement_atto_amount < 1e18`
    pub settlement_atto_amount: Uint128,
    /// Quantity expressed in natural units, `nominal_base_qty >= 0`
    pub nominal_base_qty: Uint64,
    /// Amount expressed in fractional units, `0 >= nominal_atto_qty < 1e18`
    pub nominal_atto_qty: Uint128,
    pub cu_hashes: Vec<HexBinary>,
}
