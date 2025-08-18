use bs58::decode::Error;
use core::fmt;
use cosmwasm_schema::schemars;
use cosmwasm_schema::serde::{de, ser, Deserialize, Deserializer, Serialize};
use cosmwasm_std::Binary;
use std::ops::Deref;

/// This is a wrapper around Vec<u8> to add base58 de/serialization
/// with serde. It also adds some helper methods to help encode inline.
///
/// This is similar to `cosmwasm_std::Binary` but uses base58.
/// See also <https://github.com/CosmWasm/cosmwasm/blob/main/docs/MESSAGE_TYPES.md>.
#[derive(Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord, schemars::JsonSchema)]
pub struct Base58Binary(#[schemars(with = "String")] Vec<u8>);

impl Base58Binary {
    pub fn from_base58(input: &str) -> Result<Base58Binary, Error> {
        bs58::decode(input).into_vec().map(Self)
    }

    pub fn to_base58(&self) -> String {
        bs58::encode(&self.0).into_string()
    }

    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl fmt::Debug for Base58Binary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_base58())
    }
}

/// Just like Vec<u8>, HexBinary is a smart pointer to [u8].
/// This implements `*data` for us and allows us to
/// do `&*data`, returning a `&[u8]` from a `&HexBinary`.
/// With [deref coercions](https://doc.rust-lang.org/1.22.1/book/first-edition/deref-coercions.html#deref-coercions),
/// this allows us to use `&data` whenever a `&[u8]` is required.
impl Deref for Base58Binary {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl AsRef<[u8]> for Base58Binary {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

// Slice
impl From<&[u8]> for Base58Binary {
    fn from(binary: &[u8]) -> Self {
        Self(binary.to_vec())
    }
}

// Array reference
impl<const LENGTH: usize> From<&[u8; LENGTH]> for Base58Binary {
    fn from(source: &[u8; LENGTH]) -> Self {
        Self(source.to_vec())
    }
}

// Owned array
impl<const LENGTH: usize> From<[u8; LENGTH]> for Base58Binary {
    fn from(source: [u8; LENGTH]) -> Self {
        Self(source.into())
    }
}

impl From<Vec<u8>> for Base58Binary {
    fn from(vec: Vec<u8>) -> Self {
        Self(vec)
    }
}

impl From<Base58Binary> for Vec<u8> {
    fn from(original: Base58Binary) -> Vec<u8> {
        original.0
    }
}

impl From<Binary> for Base58Binary {
    fn from(original: Binary) -> Self {
        Self(original.into())
    }
}

impl From<Base58Binary> for Binary {
    fn from(original: Base58Binary) -> Binary {
        Binary::from(original.0)
    }
}

/// Implement `Base58Binary == alloc::vec::Vec<u8>`
impl PartialEq<Vec<u8>> for Base58Binary {
    fn eq(&self, rhs: &Vec<u8>) -> bool {
        // Use Vec<u8> == Vec<u8>
        self.0 == *rhs
    }
}

/// Implement `alloc::vec::Vec<u8> == Base58Binary`
impl PartialEq<Base58Binary> for Vec<u8> {
    fn eq(&self, rhs: &Base58Binary) -> bool {
        // Use Vec<u8> == Vec<u8>
        *self == rhs.0
    }
}

/// Implement `Base58Binary == &[u8]`
impl PartialEq<&[u8]> for Base58Binary {
    fn eq(&self, rhs: &&[u8]) -> bool {
        // Use &[u8] == &[u8]
        self.as_slice() == *rhs
    }
}

/// Implement `&[u8] == Base58Binary`
impl PartialEq<Base58Binary> for &[u8] {
    fn eq(&self, rhs: &Base58Binary) -> bool {
        // Use &[u8] == &[u8]
        *self == rhs.as_slice()
    }
}

/// Implement `Base58Binary == [u8; LENGTH]`
impl<const LENGTH: usize> PartialEq<[u8; LENGTH]> for Base58Binary {
    fn eq(&self, rhs: &[u8; LENGTH]) -> bool {
        self.as_slice() == rhs.as_slice()
    }
}

/// Implement `[u8; LENGTH] == Base58Binary`
impl<const LENGTH: usize> PartialEq<Base58Binary> for [u8; LENGTH] {
    fn eq(&self, rhs: &Base58Binary) -> bool {
        self.as_slice() == rhs.as_slice()
    }
}

/// Implement `Base58Binary == &[u8; LENGTH]`
impl<const LENGTH: usize> PartialEq<&[u8; LENGTH]> for Base58Binary {
    fn eq(&self, rhs: &&[u8; LENGTH]) -> bool {
        self.as_slice() == rhs.as_slice()
    }
}

/// Implement `&[u8; LENGTH] == Base58Binary`
impl<const LENGTH: usize> PartialEq<Base58Binary> for &[u8; LENGTH] {
    fn eq(&self, rhs: &Base58Binary) -> bool {
        self.as_slice() == rhs.as_slice()
    }
}

/// Serializes as a Base58 string
impl Serialize for Base58Binary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_base58())
        } else {
            serializer.serialize_bytes(&self.0)
        }
    }
}

/// Deserializes as a Base58 string
impl<'de> Deserialize<'de> for Base58Binary {
    fn deserialize<D>(deserializer: D) -> Result<Base58Binary, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            deserializer.deserialize_str(Base58Visitor)
        } else {
            deserializer.deserialize_bytes(BytesVisitor)
        }
    }
}

struct Base58Visitor;

impl de::Visitor<'_> for Base58Visitor {
    type Value = Base58Binary;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("valid hex encoded string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match Base58Binary::from_base58(v) {
            Ok(data) => Ok(data),
            Err(_) => Err(E::custom(format!("invalid hex: {v}"))),
        }
    }
}

struct BytesVisitor;

impl de::Visitor<'_> for BytesVisitor {
    type Value = Base58Binary;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("byte array")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Base58Binary(v.to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_base58() {
        let base58 = "5HueCGU8rMjxKXQCQ2y8LeF6iE";
        let binary = Base58Binary::from_base58(base58).unwrap();
        assert_eq!(binary.to_base58(), base58);
    }

    #[test]
    fn test_to_base58() {
        let data = vec![0, 1, 2, 3, 4, 5];
        let binary = Base58Binary::from(data.clone());
        assert_eq!(binary.to_base58(), bs58::encode(&data).into_string());
    }

    #[test]
    fn test_from_various_types() {
        let vec = vec![1, 2, 3];
        let slice = &[1, 2, 3][..];
        let array = [1, 2, 3];
        let array_ref = &[1, 2, 3];

        assert_eq!(Base58Binary::from(vec.clone()), vec);
        assert_eq!(Base58Binary::from(slice), vec);
        assert_eq!(Base58Binary::from(array), vec);
        assert_eq!(Base58Binary::from(array_ref), vec);
    }

    #[test]
    fn test_serde() {
        let original = Base58Binary::from(vec![0, 1, 2, 3]);
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Base58Binary = serde_json::from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_equality() {
        let binary = Base58Binary::from(vec![1, 2, 3]);
        let vec = vec![1, 2, 3];
        let slice = &[1, 2, 3][..];
        let array = [1, 2, 3];
        let array_ref = &[1, 2, 3];

        assert_eq!(binary, vec);
        assert_eq!(binary, slice);
        assert_eq!(binary, array);
        assert_eq!(binary, array_ref);

        assert_eq!(vec, binary);
        assert_eq!(slice, binary);
        assert_eq!(array, binary);
        assert_eq!(array_ref, binary);
    }
}
