use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

// Map of ticket_hash -> exists
pub const TICKETS: Map<String, bool> = Map::new("tickets");
// Map of (user_address, block_height) -> has_burned
pub const USER_BURNS_PER_BLOCK: Map<(Addr, u64), bool> = Map::new("user_burns_per_block");
// Address of the Gratis contract for conversion
pub const GRATIS_CONTRACT: Item<Addr> = Item::new("gratis_contract");
// Admin address
pub const ADMIN: Item<Addr> = Item::new("admin");
