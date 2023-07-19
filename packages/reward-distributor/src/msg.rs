use apollo_cw_asset::AssetInfo;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{to_binary, Addr, CosmosMsg, Empty, Env, StdResult, Uint128, WasmMsg};
use cw_dex::Pool;
use cw_ownable::{cw_ownable_execute, cw_ownable_query};
use cw_vault_standard::helper::VaultContract;

#[cw_serde]
pub struct InstantiateMsg {
    /// The account to be appointed the contract owner
    pub owner: String,
    /// The emission rate per second
    pub emission_per_second: Uint128,
    /// The dex pool in which rewards are being held
    pub reward_pool: Pool,
    /// The address of the vault contract in which rewards are being held
    pub reward_vault_addr: String,
    /// The address that rewards are being distributed to
    pub distribution_addr: String,
}

#[cw_serde]
/// The internal message variants that can be called by the contract itself
pub enum InternalMsg {
    /// Callback to be called after rewards have been redeemed from the vault to send the underlying
    /// assets to the distribution address.
    VaultTokensRedeemed {},
    /// Callback to be called after LP tokens have been redeemed from the vault to send the underlying
    LpRedeemed {},
}

impl InternalMsg {
    /// Creates a CosmosMsg::Wasm::Execute message from the internal message
    pub fn into_cosmos_msg(&self, env: &Env) -> StdResult<CosmosMsg> {
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_binary(&self)?,
            funds: vec![],
        }))
    }
}

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    /// Distributes rewards to the distribution address
    Distribute {},
    /// Callback handler that can only be called by the contract itself
    Internal(InternalMsg),
}

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    /// Returns the config of the contract
    Config {},
}

#[cw_serde]
/// The config state
pub struct Config {
    /// The emission rate per second. This is the amount of tokens to be distributed per second.
    pub emission_per_second: Uint128,
    /// The denom of the vault token in which rewards are being held
    pub reward_vt_denom: String,
    /// The AssetInfo of the LP token in which rewards are being held inside the vault
    pub reward_lp_token: AssetInfo,
    /// The dex pool in which rewards are being held
    pub reward_pool: Pool,
    /// The address of the vault contract in which rewards are being held
    pub reward_vault: VaultContract<Empty, Empty>,
    /// The address that rewards are being distributed to
    pub distribution_addr: Addr,
}

#[cw_serde]
/// The response to the config query
pub struct ConfigResponse {
    /// The emission rate per second. This is the amount of tokens to be distributed per second.
    pub emission_per_second: Uint128,
    /// The denom of the vault token in which rewards are being held
    pub reward_vt_denom: String,
    /// The AssetInfo of the LP token in which rewards are being held inside the vault
    pub reward_lp_token: AssetInfo,
    /// The dex pool in which rewards are being held
    pub reward_pool: Pool,
    /// The address of the vault contract in which rewards are being held
    pub reward_vault_addr: String,
    /// The address that rewards are being distributed to
    pub distribution_addr: String,
}
