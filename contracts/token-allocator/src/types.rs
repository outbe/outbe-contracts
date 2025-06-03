use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint64;
use std::fmt;

#[cw_serde]
pub struct TokenAllocatorData {
    pub amount: Uint64,
}

impl fmt::Display for TokenAllocatorData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.amount)
    }
}
