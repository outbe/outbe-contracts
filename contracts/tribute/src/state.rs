use cw_storage_plus::Map;

// TODO decide if we need to track hashes at L1
#[allow(dead_code)]
pub const HASHES: Map<&str, String> = Map::new("hashes");
