use cosmwasm_schema::cw_serde;
use cw_dex::astroport::AstroportPool;
use cw_storage_plus::Item;
use cw_vault_standard::VaultContract;

use crate::config::Config;
use crate::ContractError;

/// An enum representing different types of reward tokens
#[cw_serde]
pub enum RewardType {
    /// The reward token is a vault token
    Vault {
        /// The vault contract
        vault: VaultContract,
        /// The Astroport pool that the vault holds liquidity in
        pool: AstroportPool,
    },
    /// The reward token is an Astroport LP token
    LP(AstroportPool),
    /// The reward token is a native coin
    Coin(String),
}

impl RewardType {
    pub fn into_pool(self) -> Result<AstroportPool, ContractError> {
        match self {
            RewardType::Vault { vault: _, pool } => Ok(pool),
            RewardType::LP(pool) => Ok(pool),
            RewardType::Coin(_) => Err(ContractError::generic_err(
                "Cannot redeem vault tokens from coin reward",
            )),
        }
    }
}

/// Stores the contract's config
pub const CONFIG: Item<Config> = Item::new("config");

/// Stores the reward token that this contract is distributing
pub const REWARD_TOKEN: Item<RewardType> = Item::new("reward_token");

/// Stores the last timestamp that rewards were distributed
pub const LAST_DISTRIBUTED: Item<u64> = Item::new("last_distributed");
