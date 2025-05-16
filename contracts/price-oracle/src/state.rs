use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Timestamp};
use cw20::Denom;
use cw_ownable::{OwnershipStore, OWNERSHIP_KEY};
use cw_storage_plus::Item;
use std::fmt;

pub const CREATOR: OwnershipStore = OwnershipStore::new(OWNERSHIP_KEY);

#[cw_serde]
pub enum DayType {
    GREEN,
    RED,
}

#[cw_serde]
pub struct TokenPairState {
    pub token1: Denom,
    pub token2: Denom,
    pub price: Decimal,
    pub day_type: DayType,
    pub last_updated: Timestamp,
}

pub const TOKEN_PAIR_PRICE: Item<TokenPairState> = Item::new("token_pair");

impl fmt::Display for DayType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DayType::GREEN => write!(f, "GREEN"),
            DayType::RED => write!(f, "RED"),
        }
    }
}
