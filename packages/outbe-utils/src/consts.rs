use cosmwasm_std::{Decimal, Uint128};

/// Decimal precision for Native coin.
pub const DECIMAL_PLACES: u32 = 18;

/// Decimal precision for Native coin.
pub const DECIMALS: Uint128 = Uint128::new(1_000_000_000_000_000_000u128);

pub fn to_decimals_amount(amount: Uint128) -> Decimal {
    Decimal::from_atomics(amount, DECIMAL_PLACES).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_to_decimals_amount() {
        let amount = Uint128::new(1_000_000);
        let result = to_decimals_amount(amount);
        assert_eq!(result, Decimal::from_str("0.000000000001").unwrap());
    }

    #[test]
    fn test_to_decimals_amount_zero() {
        let amount = Uint128::zero();
        let result = to_decimals_amount(amount);
        assert_eq!(result, Decimal::zero());
    }

    #[test]
    fn test_to_decimals_amount_max() {
        let amount = Uint128::MAX;
        let result = to_decimals_amount(amount);
        assert_eq!(
            result,
            Decimal::from_atomics(amount, DECIMAL_PLACES).unwrap()
        );
    }

    #[test]
    fn test_decimals() {
        assert_eq!(DECIMALS, Decimal::one().atomics());
    }
}
