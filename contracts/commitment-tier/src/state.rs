use crate::types::CommitmentTier;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;
use cw_ownable::{OwnershipStore, OWNERSHIP_KEY};
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub tiers: Vec<CommitmentTier>,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const CREATOR: OwnershipStore = OwnershipStore::new(OWNERSHIP_KEY);

// TODO specify a correct weights
pub fn default_tiers() -> Vec<CommitmentTier> {
    vec![
        CommitmentTier {
            tier_id: 1,
            tpt_percent_increase: Uint128::new(0u128),
            weight: Uint128::new(1),
        },
        CommitmentTier {
            tier_id: 2,
            tpt_percent_increase: Uint128::new(8u128),
            weight: Uint128::new(1),
        },
        CommitmentTier {
            tier_id: 3,
            tpt_percent_increase: Uint128::new(16u128),
            weight: Uint128::new(1),
        },
        CommitmentTier {
            tier_id: 4,
            tpt_percent_increase: Uint128::new(24u128),
            weight: Uint128::new(1),
        },
        CommitmentTier {
            tier_id: 5,
            tpt_percent_increase: Uint128::new(32u128),
            weight: Uint128::new(1),
        },
        CommitmentTier {
            tier_id: 6,
            tpt_percent_increase: Uint128::new(40u128),
            weight: Uint128::new(1),
        },
        CommitmentTier {
            tier_id: 7,
            tpt_percent_increase: Uint128::new(48u128),
            weight: Uint128::new(1),
        },
        CommitmentTier {
            tier_id: 8,
            tpt_percent_increase: Uint128::new(56u128),
            weight: Uint128::new(1),
        },
        CommitmentTier {
            tier_id: 9,
            tpt_percent_increase: Uint128::new(64u128),
            weight: Uint128::new(1),
        },
        CommitmentTier {
            tier_id: 10,
            tpt_percent_increase: Uint128::new(72u128),
            weight: Uint128::new(1),
        },
        CommitmentTier {
            tier_id: 11,
            tpt_percent_increase: Uint128::new(80u128),
            weight: Uint128::new(1),
        },
        CommitmentTier {
            tier_id: 12,
            tpt_percent_increase: Uint128::new(88u128),
            weight: Uint128::new(1),
        },
        CommitmentTier {
            tier_id: 13,
            tpt_percent_increase: Uint128::new(96u128),
            weight: Uint128::new(1),
        },
        CommitmentTier {
            tier_id: 14,
            tpt_percent_increase: Uint128::new(104u128),
            weight: Uint128::new(1),
        },
        CommitmentTier {
            tier_id: 15,
            tpt_percent_increase: Uint128::new(112u128),
            weight: Uint128::new(1),
        },
        CommitmentTier {
            tier_id: 16,
            tpt_percent_increase: Uint128::new(120u128),
            weight: Uint128::new(1),
        },
    ]
}
