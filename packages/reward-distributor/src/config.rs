use apollo_cw_asset::AssetInfoBase;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, Empty, StdResult, Uint128};
use cw_address_like::AddressLike;
use cw_dex::Pool;
use cw_vault_standard::VaultContract;
use derive_builder::Builder;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cw_serde]
#[derive(Builder)]
#[builder(derive(Serialize, Deserialize, Debug, PartialEq, JsonSchema))]
/// The config state
pub struct ConfigBase<T: AddressLike> {
    /// The emission rate per second. This is the amount of tokens to be
    /// distributed per second.
    pub emission_per_second: Uint128,
    /// The denom of the vault token in which rewards are being held
    pub reward_vt_denom: String,
    /// The AssetInfo of the LP token in which rewards are being held inside the
    /// vault
    pub reward_lp_token: AssetInfoBase<T>,
    /// The dex pool in which rewards are being held
    pub reward_pool: Pool,
    /// The address of the vault contract in which rewards are being held
    pub reward_vault: VaultContract<Empty, Empty>,
    /// The address that rewards are being distributed to
    pub distribution_addr: T,
}

pub type ConfigUnchecked = ConfigBase<String>;
pub type Config = ConfigBase<Addr>;
pub type ConfigUpdates = ConfigBaseBuilder<String>;

impl ConfigUnchecked {
    /// Checks that the `ConfigUnchecked` is valid and returns a `Config`
    pub fn check(self, api: &dyn Api) -> StdResult<Config> {
        Ok(Config {
            emission_per_second: self.emission_per_second,
            reward_vt_denom: self.reward_vt_denom,
            reward_lp_token: self.reward_lp_token.check(api)?,
            reward_pool: self.reward_pool,
            reward_vault: self.reward_vault,
            distribution_addr: api.addr_validate(&self.distribution_addr)?,
        })
    }
}

impl Config {
    /// Updates the existing config with the given updates. If a field is
    /// `None` in the `updates` then the old config is kept, else it is updated
    /// to the new value.
    pub fn update(self, api: &dyn Api, updates: ConfigUpdates) -> StdResult<Config> {
        ConfigUnchecked {
            emission_per_second: updates
                .emission_per_second
                .unwrap_or_else(|| self.emission_per_second),
            reward_vt_denom: updates
                .reward_vt_denom
                .unwrap_or_else(|| self.reward_vt_denom),
            reward_lp_token: updates
                .reward_lp_token
                .unwrap_or_else(|| self.reward_lp_token.into()),
            reward_pool: updates.reward_pool.unwrap_or_else(|| self.reward_pool),
            reward_vault: updates.reward_vault.unwrap_or_else(|| self.reward_vault),
            distribution_addr: updates
                .distribution_addr
                .unwrap_or_else(|| self.distribution_addr.into()),
        }
        .check(api)
    }
}
