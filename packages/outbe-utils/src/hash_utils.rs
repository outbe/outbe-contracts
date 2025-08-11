use blake3::Hasher;
use cosmwasm_std::HexBinary;

pub fn generate_hash_id(prefix: &str, fields: Vec<&[u8]>) -> HexBinary {
    let mut hasher = Hasher::new();
    hasher.update(prefix.as_bytes());

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
        let hash = generate_hash_id(prefix, fields.clone());

        // Hash should not be empty
        assert!(!hash.is_empty());

        // Same inputs should generate same hash
        let hash2 = generate_hash_id(prefix, fields.clone());
        assert_eq!(hash, hash2);

        // Different prefix should generate different hash
        let hash3 = generate_hash_id("other", fields.clone());
        assert_ne!(hash, hash3);

        // Different fields should generate different hash
        let fields2 = vec!["field1".as_bytes(), "field3".as_bytes()];
        let hash4 = generate_hash_id(prefix, fields2);
        assert_ne!(hash, hash4);

        // Empty fields should work
        let empty_fields: Vec<&[u8]> = vec![];
        let hash5 = generate_hash_id(prefix, empty_fields);
        assert!(!hash5.is_empty());
    }
}
