use cw_dex::astroport::AstroportPool;
use cw_storage_plus::Item;
use cw_vault_standard::VaultContract;

use crate::config::Config;

/// Stores the contract's config
pub const CONFIG: Item<Config> = Item::new("config");

/// Stores the Astroport pool in which rewards are being held
pub const REWARD_POOL: Item<AstroportPool> = Item::new("reward_pool");

/// Stores the vault contract in which rewards are being held
pub const REWARD_VAULT: Item<VaultContract> = Item::new("reward_vault");

/// Stores the last timestamp that rewards were distributed
pub const LAST_DISTRIBUTED: Item<u64> = Item::new("last_distributed");
