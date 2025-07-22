use cw_ownable::{OwnershipStore, OWNERSHIP_KEY};
use cw_storage_plus::Item;

pub const CREATOR: OwnershipStore = OwnershipStore::new(OWNERSHIP_KEY);

pub const SEED: Item<u64> = Item::new("seed");
