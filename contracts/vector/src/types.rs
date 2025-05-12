use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Uint128};

#[cw_serde]
pub struct Vector {
    /// Vector identifier
    pub vector_id: u16,
    /// Name or label of the vector tier
    pub name: String,
    /// TPT+%
    pub performance_rate: Uint128,
    /// Winning Probability weight
    pub weight: Decimal,
}
