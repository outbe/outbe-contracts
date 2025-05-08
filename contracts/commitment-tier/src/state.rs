use crate::types::CommitmentTier;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Uint128};
use cw_ownable::{OwnershipStore, OWNERSHIP_KEY};
use cw_storage_plus::Item;
use std::str::FromStr;

#[cw_serde]
pub struct Config {
    pub tiers: Vec<CommitmentTier>,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const CREATOR: OwnershipStore = OwnershipStore::new(OWNERSHIP_KEY);

/// Defined a list of commitment tiers with their weights.
/// The base formula for weights is the following:
/// `weight = (1/8) ^ (16 - n / 15)` where n is id of the tier
pub fn default_tiers() -> Vec<CommitmentTier> {
    vec![
        CommitmentTier {
            tier_id: 1,
            performance_rate: Uint128::new(0u128),
            weight: Decimal::from_str("0.125").unwrap(),
        },
        CommitmentTier {
            tier_id: 2,
            performance_rate: Uint128::new(8u128),
            weight: Decimal::from_str("0.14358729").unwrap(),
        },
        CommitmentTier {
            tier_id: 3,
            performance_rate: Uint128::new(16u128),
            weight: Decimal::from_str("0.16493849").unwrap(),
        },
        CommitmentTier {
            tier_id: 4,
            performance_rate: Uint128::new(24u128),
            weight: Decimal::from_str("0.18946457").unwrap(),
        },
        CommitmentTier {
            tier_id: 5,
            performance_rate: Uint128::new(32u128),
            weight: Decimal::from_str("0.21763764").unwrap(),
        },
        CommitmentTier {
            tier_id: 6,
            performance_rate: Uint128::new(40u128),
            weight: Decimal::from_str("0.25").unwrap(),
        },
        CommitmentTier {
            tier_id: 7,
            performance_rate: Uint128::new(48u128),
            weight: Decimal::from_str("0.28717459").unwrap(),
        },
        CommitmentTier {
            tier_id: 8,
            performance_rate: Uint128::new(56u128),
            weight: Decimal::from_str("0.32987698").unwrap(),
        },
        CommitmentTier {
            tier_id: 9,
            performance_rate: Uint128::new(64u128),
            weight: Decimal::from_str("0.37892914").unwrap(),
        },
        CommitmentTier {
            tier_id: 10,
            performance_rate: Uint128::new(72u128),
            weight: Decimal::from_str("0.43527528").unwrap(),
        },
        CommitmentTier {
            tier_id: 11,
            performance_rate: Uint128::new(80u128),
            weight: Decimal::from_str("0.5").unwrap(),
        },
        CommitmentTier {
            tier_id: 12,
            performance_rate: Uint128::new(88u128),
            weight: Decimal::from_str("0.57434918").unwrap(), //0,
        },
        CommitmentTier {
            tier_id: 13,
            performance_rate: Uint128::new(96u128),
            weight: Decimal::from_str("0.65975396").unwrap(),
        },
        CommitmentTier {
            tier_id: 14,
            performance_rate: Uint128::new(104u128),
            weight: Decimal::from_str("0.75785828").unwrap(),
        },
        CommitmentTier {
            tier_id: 15,
            performance_rate: Uint128::new(112u128),
            weight: Decimal::from_str("0.75785828").unwrap(),
        },
        CommitmentTier {
            tier_id: 16,
            performance_rate: Uint128::new(120u128),
            weight: Decimal::one(),
        },
    ]
}
