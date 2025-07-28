use cosmwasm_std::{Decimal, Uint128};

/// Decimal precision for Native coin.
pub const DECIMAL_PLACES: u32 = 18;

/// Decimal precision for Native coin.
pub const DECIMALS: Uint128 = Uint128::new(10 ^ DECIMAL_PLACES as u128);

pub fn to_decimals_amount(amount: Uint128) -> Decimal {
    Decimal::from_atomics(amount, DECIMAL_PLACES).unwrap()
}
