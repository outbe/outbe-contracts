use cw_ownable::{OwnershipStore, OWNERSHIP_KEY};

pub const CREATOR: OwnershipStore = OwnershipStore::new(OWNERSHIP_KEY);
