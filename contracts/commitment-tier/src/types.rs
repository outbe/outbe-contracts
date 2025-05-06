use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Uint128};

/// Commitment tier public data
#[cw_serde]
pub struct CommitmentTier {
    /// Identifier
    pub tier_id: u16,
    /// TPT+%
    pub tpt_percent_increase: Uint128,
    /// Winning Probability weight
    pub weight: Decimal,
}
