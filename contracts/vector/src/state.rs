use crate::types::Vector;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Uint128};
use cw_ownable::{OwnershipStore, OWNERSHIP_KEY};
use cw_storage_plus::Item;
use std::str::FromStr;

#[cw_serde]
pub struct Config {
    pub vectors: Vec<Vector>,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const CREATOR: OwnershipStore = OwnershipStore::new(OWNERSHIP_KEY);

/// Defined a list of vector tiers with their weights.
/// The base formula for weights is the following:
/// `weight = (1/8) ^ (16 - n / 15)` where n is id of the tier
pub fn default_vector_tiers() -> Vec<Vector> {
    vec![
        Vector {
            vector_id: 1,
            name: "Growth Vector 8%".to_string(),
            performance_rate: Uint128::new(8u128),
            weight: Decimal::from_str("0.125").unwrap(),
        },
        Vector {
            vector_id: 2,
            name: "Growth Vector 16%".to_string(),
            performance_rate: Uint128::new(16u128),
            weight: Decimal::from_str("0.14358729").unwrap(),
        },
        Vector {
            vector_id: 3,
            name: "Growth Vector 24%".to_string(),
            performance_rate: Uint128::new(24u128),
            weight: Decimal::from_str("0.16493849").unwrap(),
        },
        Vector {
            vector_id: 4,
            name: "Growth Vector 32%".to_string(),
            performance_rate: Uint128::new(32u128),
            weight: Decimal::from_str("0.18946457").unwrap(),
        },
        Vector {
            vector_id: 5,
            name: "Growth Vector 40%".to_string(),
            performance_rate: Uint128::new(40u128),
            weight: Decimal::from_str("0.21763764").unwrap(),
        },
        Vector {
            vector_id: 6,
            name: "Growth Vector 48%".to_string(),
            performance_rate: Uint128::new(48u128),
            weight: Decimal::from_str("0.25").unwrap(),
        },
        Vector {
            vector_id: 7,
            name: "Growth Vector 56%".to_string(),
            performance_rate: Uint128::new(56u128),
            weight: Decimal::from_str("0.28717459").unwrap(),
        },
        Vector {
            vector_id: 8,
            name: "Growth Vector 64%".to_string(),
            performance_rate: Uint128::new(64u128),
            weight: Decimal::from_str("0.32987698").unwrap(),
        },
        Vector {
            vector_id: 9,
            name: "Growth Vector 72%".to_string(),
            performance_rate: Uint128::new(72u128),
            weight: Decimal::from_str("0.37892914").unwrap(),
        },
        Vector {
            vector_id: 10,
            name: "Growth Vector 80%".to_string(),
            performance_rate: Uint128::new(80u128),
            weight: Decimal::from_str("0.43527528").unwrap(),
        },
        Vector {
            vector_id: 11,
            name: "Growth Vector 88%".to_string(),
            performance_rate: Uint128::new(88u128),
            weight: Decimal::from_str("0.5").unwrap(),
        },
        Vector {
            vector_id: 12,
            name: "Growth Vector 96%".to_string(),
            performance_rate: Uint128::new(96u128),
            weight: Decimal::from_str("0.57434918").unwrap(), //0,
        },
        Vector {
            vector_id: 13,
            name: "Growth Vector 104%".to_string(),
            performance_rate: Uint128::new(104u128),
            weight: Decimal::from_str("0.65975396").unwrap(),
        },
        Vector {
            vector_id: 14,
            name: "Growth Vector 112%".to_string(),
            performance_rate: Uint128::new(112u128),
            weight: Decimal::from_str("0.75785828").unwrap(),
        },
        Vector {
            vector_id: 15,
            name: "Growth Vector 120%".to_string(),
            performance_rate: Uint128::new(120u128),
            weight: Decimal::from_str("0.75785828").unwrap(),
        },
        Vector {
            vector_id: 16,
            name: "Growth Vector 128%".to_string(),
            performance_rate: Uint128::new(128u128),
            weight: Decimal::one(),
        },
    ]
}
