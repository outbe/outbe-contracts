use cosmwasm_std::Uint128;
use thiserror::Error;

pub const FRACTIONAL: Uint128 = Uint128::new(1000000000000000000u128); // 1*10**18

#[derive(Error, Debug)]
pub enum AmountError {
    #[error("Wrong Atto amount")]
    WrongAtto {},
}

pub fn normalize_amount(base: u64, atto: u64) -> Result<Uint128, AmountError> {
    let atto128 = Uint128::from(atto);
    if atto128 >= FRACTIONAL {
        return Err(AmountError::WrongAtto {});
    }
    Ok(Uint128::from(base) * FRACTIONAL + atto128)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_normalize_amount() {
        // Test with non-zero base and atto
        assert_eq!(
            normalize_amount(1, 123456789012345678).unwrap(),
            Uint128::new(1_123456789012345678)
        );

        // Test with zero atto
        assert_eq!(
            normalize_amount(5, 0).unwrap(),
            Uint128::new(5_000000000000000000)
        );

        // Test with dot precise
        assert_eq!(
            normalize_amount(0, 500000000000000000).unwrap(),
            Decimal::from_str("0.5").unwrap().atomics()
        );
        assert_eq!(
            normalize_amount(100500, 500000000000000000).unwrap(),
            Decimal::from_str("100500.5").unwrap().atomics()
        );

        // Test with zero base
        assert_eq!(normalize_amount(0, 123).unwrap(), Uint128::new(123));

        // Test with zero for both base and atto
        assert_eq!(normalize_amount(0, 0).unwrap(), Uint128::zero());

        // Test boundary condition: atto is one less than PRECISE
        assert_eq!(
            normalize_amount(1, 999_999_999_999_999_999).unwrap(),
            Uint128::new(1_999_999_999_999_999_999)
        );
    }

    #[test]
    fn test_normalize_amount_error() {
        // Test boundary condition: atto is equal to PRECISE
        let result = normalize_amount(1, 1_000_000_000_000_000_000);
        assert!(matches!(result, Err(AmountError::WrongAtto {})));

        // Test with atto greater than PRECISE
        let result = normalize_amount(1, 1_000_000_000_000_000_001);
        assert!(matches!(result, Err(AmountError::WrongAtto {})));

        // Test with max u64 value for atto, which is greater than PRECISE
        let result = normalize_amount(1, u64::MAX);
        assert!(matches!(result, Err(AmountError::WrongAtto {})));
    }
}
