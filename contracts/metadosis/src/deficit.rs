use cosmwasm_std::{Decimal, Uint128};
use outbe_utils::consts::to_decimals_amount;
use std::f64::consts::E;

// Total Lysis Deficit is calculated as the maximum of
// (Total Tribute Interest - Total Lysis Limit) or 32% of Total Tribute Interest.
pub(crate) fn calc_total_deficit(
    total_tribute_interest: Uint128,
    total_lysis_limit: Uint128,
    deficit_percent: Decimal,
) -> Uint128 {
    let mut total_deficit =
        (deficit_percent * to_decimals_amount(total_tribute_interest)).atomics();

    if total_tribute_interest > total_lysis_limit
        && total_tribute_interest - total_lysis_limit > total_deficit
    {
        total_deficit = total_tribute_interest - total_lysis_limit;
    }
    total_deficit
}

/// Returns lysis deficit for each lysis run i.e. len = 23
pub(crate) fn calc_lysis_deficits(total_deficit: Uint128) -> Vec<Uint128> {
    let mut deficits: Vec<Uint128> = Vec::with_capacity(23);

    let total_deficit_f = total_deficit.u128() as f64;

    // Calculate a denominator sum first
    let mut denominator: f64 = 0.0;
    for j in 1..=23 {
        let pwr: f64 = -0.2 * (j - 1) as f64;
        denominator += E.powf(pwr);
    }

    // Calculate individual deficits
    for r in 1..=23 {
        let pwr: f64 = -0.2 * (r - 1) as f64;
        let numerator = E.powf(pwr);
        let deficit = total_deficit_f * numerator / denominator;
        deficits.push(Uint128::new(deficit as u128));
    }
    // NB: reverse order because lysis starts from 23
    deficits.reverse();

    deficits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_lysis_deficits_zero() {
        let deficits = calc_lysis_deficits(Uint128::zero());
        assert_eq!(deficits.len(), 23);
        assert!(deficits.iter().all(|x| x.is_zero()));
    }

    #[test]
    fn test_calc_lysis_deficits() {
        let total = Uint128::new(1000000);
        let deficits = calc_lysis_deficits(total);

        assert_eq!(deficits.len(), 23);

        // Sum of all deficits should approximately equal total
        let sum: u128 = deficits.iter().map(|x| x.u128()).sum();
        assert!(sum >= total.u128() * 99 / 100); // Allow 1% error due to rounding
        assert!(sum <= total.u128());

        // Each next deficit should be more than the previous (because of reverse order)
        for i in 1..deficits.len() {
            assert!(deficits[i] > deficits[i - 1]);
        }
    }

    #[test]
    fn test_calc_lysis_deficits_rounding() {
        let total = Uint128::new(7); // Small number to force rounding
        let deficits = calc_lysis_deficits(total);

        assert_eq!(deficits.len(), 23);

        // Sum should not exceed the original amount
        let sum: u128 = deficits.iter().map(|x| x.u128()).sum();
        assert!(sum <= total.u128());
    }
}
