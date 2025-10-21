use crate::types::{PriceData, VwapData};
use cosmwasm_std::{Decimal, Timestamp, Uint128};
use outbe_utils::denom::Denom;

/// Generate a consistent pair ID from two tokens
/// Always orders tokens alphabetically to ensure consistency
pub fn get_pair_id(token1: &Denom, token2: &Denom) -> String {
    let token1_str = token1.to_string();
    let token2_str = token2.to_string();

    if token1_str < token2_str {
        format!("{}-{}", token1_str, token2_str)
    } else {
        format!("{}-{}", token2_str, token1_str)
    }
}

/// Calculate VWAP from price history
pub fn calculate_vwap(
    price_history: &[PriceData],
    current_time: Timestamp,
    window_seconds: u64,
) -> Option<VwapData> {
    if price_history.is_empty() {
        return None;
    }

    let window_start = current_time.seconds().saturating_sub(window_seconds);

    let mut total_value = Uint128::zero();
    let mut total_volume = Uint128::zero();

    for price_data in price_history {
        if price_data.timestamp.seconds() >= window_start {
            if let Some(volume) = price_data.volume {
                if !volume.is_zero() {
                    let price_u128 = (price_data.price * Decimal::from_ratio(1_000_000u128, 1u128))
                        .to_uint_floor();
                    let value = price_u128.checked_mul(volume).unwrap_or(Uint128::zero());
                    total_value = total_value.checked_add(value).unwrap_or(total_value);
                    total_volume = total_volume.checked_add(volume).unwrap_or(total_volume);
                }
            }
        }
    }

    if total_volume.is_zero() {
        return None;
    }

    let vwap =
        Decimal::from_ratio(total_value, total_volume) / Decimal::from_ratio(1_000_000u128, 1u128);

    Some(VwapData {
        vwap,
        total_volume,
        window_seconds,
        timestamp: current_time,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use outbe_utils::denom::Denom;

    #[test]
    fn test_get_pair_id() {
        let token1 = Denom::Native("coen".to_string());
        let token2 = Denom::Native("usdc".to_string());

        // Should be consistent regardless of order
        assert_eq!(get_pair_id(&token1, &token2), "native_coen-native_usdc");
        assert_eq!(get_pair_id(&token2, &token1), "native_coen-native_usdc");
    }
}
