use cw_storage_plus::Item;

use crate::config::Config;

/// Stores the contract's config
pub const CONFIG: Item<Config> = Item::new("config");

/// Stores the last timestamp that rewards were distributed
pub const LAST_DISTRIBUTED: Item<u64> = Item::new("last_distributed");
