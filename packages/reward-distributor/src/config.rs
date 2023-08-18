use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, StdResult, Uint128};
use cw_address_like::AddressLike;
use derive_builder::Builder;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cw_serde]
#[derive(Builder)]
#[builder(derive(Serialize, Deserialize, Debug, PartialEq, JsonSchema))]
/// The config state
pub struct ConfigBase<T: AddressLike> {
    /// The emission rate per second. This is the amount of tokens to be
    /// distributed per second, denominated in vault tokens of the reward vault.
    pub emission_per_second: Uint128,
    /// The address that rewards are being distributed to
    pub distribution_addr: T,
    /// The unix timestamp at which rewards start being distributed
    pub rewards_start_time: u64,
}

pub type ConfigUnchecked = ConfigBase<String>;
pub type Config = ConfigBase<Addr>;
pub type ConfigUpdates = ConfigBaseBuilder<String>;

impl ConfigUnchecked {
    /// Checks that the `ConfigUnchecked` is valid and returns a `Config`
    pub fn check(self, api: &dyn Api) -> StdResult<Config> {
        Ok(Config {
            emission_per_second: self.emission_per_second,
            distribution_addr: api.addr_validate(&self.distribution_addr)?,
            rewards_start_time: self.rewards_start_time,
        })
    }
}

impl Config {
    /// Updates the existing config with the given updates. If a field is
    /// `None` in the `updates` then the old config is kept, else it is updated
    /// to the new value.
    pub fn update(&self, api: &dyn Api, updates: ConfigUpdates) -> StdResult<Config> {
        ConfigUnchecked {
            emission_per_second: updates
                .emission_per_second
                .unwrap_or(self.emission_per_second),
            distribution_addr: updates
                .distribution_addr
                .unwrap_or_else(|| self.distribution_addr.clone().into()),
            rewards_start_time: updates
                .rewards_start_time
                .unwrap_or(self.rewards_start_time),
        }
        .check(api)
    }
}
