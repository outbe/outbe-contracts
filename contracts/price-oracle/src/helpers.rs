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

#[cfg(test)]
mod tests {
    use super::*;
    use outbe_utils::denom::Denom;

    #[test]
    fn test_get_pair_id() {
        let token1 = Denom::Native("uatom".to_string());
        let token2 = Denom::Native("uosmo".to_string());

        // Should be consistent regardless of order
        assert_eq!(get_pair_id(&token1, &token2), "native_uatom-native_uosmo");
        assert_eq!(get_pair_id(&token2, &token1), "native_uatom-native_uosmo");
    }
}
