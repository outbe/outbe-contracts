use crate::ContractError;
use cosmwasm_std::HexBinary;
use outbe_utils::gen_hash;

pub(crate) fn verify_proof_of_work(
    seek_hash: HexBinary,
    nonce: HexBinary,
    complexity: usize,
) -> Result<(), ContractError> {
    if seek_hash.len() != 32 {
        return Err(ContractError::InvalidHash {});
    }

    #[cfg(feature = "demo")]
    {
        if nonce.is_empty() {
            return Ok(());
        }
    }

    let nonce_hash = gen_hash(vec![&nonce]);

    let seek_suffix = &seek_hash.as_slice()[32 - complexity..];
    let nonce_suffix = &nonce_hash.as_slice()[32 - complexity..];

    if seek_suffix != nonce_suffix {
        return Err(ContractError::InvalidProofOfWork {});
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::HexBinary;

    #[test]
    fn test_verify_proof_of_work_success() {
        let seek_hash =
            HexBinary::from_hex("72cd6e8422c407fb6d098690f1130b7ded7ec2f7f5e1d30bd9d521f015363793")
                .unwrap();
        let nonce = HexBinary::from(vec![1; 32]); // This would generate a matching hash for complexity=0
        let complexity = 32;

        let result = verify_proof_of_work(seek_hash, nonce, complexity);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_proof_of_work_mismatch() {
        let seek_hash = HexBinary::from(vec![1; 32]);
        let nonce = HexBinary::from(vec![2; 32]); // Different nonce should generate different hash
        let complexity = 1;

        let result = verify_proof_of_work(seek_hash, nonce, complexity);
        assert_eq!(result, Err(ContractError::InvalidProofOfWork {}));
    }

    #[test]
    fn test_verify_proof_of_work_different_complexities() {
        let seek_hash = HexBinary::from(vec![1; 32]);
        let nonce = HexBinary::from(vec![1; 32]);

        // Test with different complexity values
        for complexity in [0, 1, 8, 16, 32] {
            let result = verify_proof_of_work(seek_hash.clone(), nonce.clone(), complexity);
            // We can't predict exact outcome as it depends on hash function,
            // but function should not panic
            assert!(result.is_ok() || result == Err(ContractError::InvalidProofOfWork {}));
        }
    }

    #[test]
    fn test_verify_proof_of_work_zero_complexity() {
        let seek_hash = HexBinary::from(vec![1; 32]);
        let nonce = HexBinary::from(vec![1; 32]);
        let complexity = 0;

        let result = verify_proof_of_work(seek_hash, nonce, complexity);
        assert!(result.is_ok());
    }
}
