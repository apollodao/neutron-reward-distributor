use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{to_binary, CosmosMsg, Env, StdResult, Uint128, WasmMsg};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};

use crate::{Config, ConfigUpdates, RewardType};

/// An enum for the information needed to instantiate the contract depending on
/// the type of reward token used.
#[cw_serde]
pub enum RewardInfo {
    /// The address of the vault if the reward token is a vault token
    VaultAddr(String),
    /// The address of the Astroport pool if the reward token is an Astroport LP
    /// token
    AstroportPoolAddr(String),
    /// The denom of the native coin if the reward token is a native coin
    NativeCoin(String),
}

#[cw_serde]
pub struct InstantiateMsg {
    /// The account to be appointed the contract owner
    pub owner: String,
    /// The emission rate per second
    pub emission_per_second: Uint128,
    /// The info needed to instantiate the contract depending on the type of
    /// reward token used
    pub reward_token_info: RewardInfo,
    /// The address that rewards are being distributed to
    pub distribution_addr: String,
    /// The unix timestamp at which rewards start being distributed
    pub rewards_start_time: u64,
}

#[cw_serde]
/// The internal message variants that can be called by the contract itself
pub enum InternalMsg {
    /// Callback to be called after rewards have been redeemed from the vault to
    /// send the underlying assets to the distribution address.
    VaultTokensRedeemed {},
    /// Callback to be called after LP tokens have been redeemed from the vault
    /// to send the underlying
    LpRedeemed {},
}

impl InternalMsg {
    /// Creates a CosmosMsg::Wasm::Execute message from the internal message
    pub fn into_cosmos_msg(&self, env: &Env) -> StdResult<CosmosMsg> {
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_binary(&ExecuteMsg::Internal(self.clone()))?,
            funds: vec![],
        }))
    }
}

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    /// Distributes rewards to the distribution address
    Distribute {},
    /// Update the contract's config
    UpdateConfig { updates: ConfigUpdates },
    /// Callback handler that can only be called by the contract itself
    Internal(InternalMsg),
}

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(StateResponse)]
    /// Returns the config of the contract as well as non-configurable contract
    /// state
    State {},
}

#[cw_serde]
/// The response to a config query
pub struct StateResponse {
    pub config: Config,
    pub reward_token: RewardType,
    pub last_distributed: u64,
}
