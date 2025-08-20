use cosmwasm_std::HexBinary;
use sha2::{Digest, Sha256};

pub fn gen_compound_hash(prefix: Option<&str>, fields: Vec<&[u8]>) -> HexBinary {
    let mut hasher = prefix
        .map(|p| Sha256::new_with_prefix(p.as_bytes()))
        .unwrap_or_default();

    for field in fields.into_iter() {
        hasher.update(b":");
        hasher.update(field);
    }
    let hash_bytes: [u8; 32] = hasher.finalize().into();
    HexBinary::from(hash_bytes.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_hash_id() {
        let prefix = "test";
        let fields = vec!["field1".as_bytes(), "field2".as_bytes()];
        let hash = gen_compound_hash(Some(prefix), fields.clone());

        // Hash should not be empty
        assert!(!hash.is_empty());

        // Same inputs should generate same hash
        let hash2 = gen_compound_hash(Some(prefix), fields.clone());
        assert_eq!(hash, hash2);

        // Different prefix should generate different hash
        let hash3 = gen_compound_hash(Some("other"), fields.clone());
        assert_ne!(hash, hash3);

        // Different fields should generate different hash
        let fields2 = vec!["field1".as_bytes(), "field3".as_bytes()];
        let hash4 = gen_compound_hash(Some(prefix), fields2);
        assert_ne!(hash, hash4);

        // Empty fields should work
        let empty_fields: Vec<&[u8]> = vec![];
        let hash5 = gen_compound_hash(Some(prefix), empty_fields);
        assert!(!hash5.is_empty());
    }
}
