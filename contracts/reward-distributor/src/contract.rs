use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, QueryRequest, Response,
    StdError, StdResult, WasmQuery,
};
use cw20::{Cw20QueryMsg, MinterResponse};
use cw_dex::astroport::AstroportPool;
use cw_vault_standard::{VaultContract, VaultContractUnchecked};
use neutron_astroport_reward_distributor::{
    Config, ConfigUnchecked, ContractError, ExecuteMsg, InstantiateMsg, InternalMsg, QueryMsg,
    RewardInfo, RewardType, StateResponse, CONFIG, LAST_DISTRIBUTED, REWARD_TOKEN,
};

use crate::execute;

pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(&msg.owner))?;

    let reward_token = match msg.reward_token_info {
        RewardInfo::AstroportVault(astroport_vault) => {
            let reward_vault: VaultContract =
                VaultContractUnchecked::new(&astroport_vault.vault_addr).check(deps.api)?;

            // Validate reward vault base token as CW20 Astroport LP token
            let reward_lp_token = deps
                .api
                .addr_validate(&reward_vault.query_vault_info(&deps.querier)?.base_token)
                .map_err(|_| StdError::generic_err("Invalid base token of reward vault"))?;

            // Query minter of LP token to get reward pool address
            let minter_res: MinterResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: reward_lp_token.to_string(),
                    msg: to_json_binary(&Cw20QueryMsg::Minter {})?,
                }))?;
            let reward_pool_addr = deps.api.addr_validate(&minter_res.minter)?;

            // Query reward pool for pool info to create pool object
            let reward_pool = AstroportPool::new(
                deps.as_ref(),
                reward_pool_addr,
                deps.api
                    .addr_validate(&astroport_vault.liquidity_manager_addr)?,
            )?;

            RewardType::Vault {
                vault: reward_vault,
                pool: reward_pool,
            }
        }
        RewardInfo::AstroportPool(astroport_pool) => {
            let reward_pool = AstroportPool::new(
                deps.as_ref(),
                deps.api.addr_validate(&astroport_pool.pool_addr)?,
                deps.api
                    .addr_validate(&astroport_pool.liquidity_manager_addr)?,
            )?;

            RewardType::LP(reward_pool)
        }
        RewardInfo::NativeCoin(reward_coin_denom) => RewardType::Coin(reward_coin_denom),
    };

    // Create config
    let config: Config = ConfigUnchecked {
        distribution_addr: msg.distribution_addr,
        emission_per_second: msg.emission_per_second,
        rewards_start_time: msg.rewards_start_time,
    }
    .check(deps.api)?;

    CONFIG.save(deps.storage, &config)?;
    LAST_DISTRIBUTED.save(deps.storage, &env.block.time.seconds())?;
    REWARD_TOKEN.save(deps.storage, &reward_token)?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateOwnership(action) => {
            let ownership = cw_ownable::update_ownership(deps, &env.block, &info.sender, action)?;
            Ok(Response::default().add_attributes(ownership.into_attributes()))
        }
        ExecuteMsg::Distribute {} => execute::execute_distribute(deps, env),
        ExecuteMsg::UpdateConfig { updates } => {
            execute::execute_update_config(deps, env, info, updates)
        }
        ExecuteMsg::Internal(msg) => {
            // Internal messages can only be called by the contract itself
            if info.sender != env.contract.address {
                return Err(ContractError::Unauthorized {});
            }

            match msg {
                InternalMsg::VaultTokensRedeemed {} => {
                    execute::execute_internal_vault_tokens_redeemed(deps.as_ref(), env)
                }
                InternalMsg::LpRedeemed {} => {
                    execute::execute_internal_lp_redeemed(deps.as_ref(), env)
                }
            }
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => {
            let ownership = cw_ownable::get_ownership(deps.storage)?;
            to_json_binary(&ownership)
        }
        QueryMsg::State {} => {
            let config = CONFIG.load(deps.storage)?;
            let last_distributed = LAST_DISTRIBUTED.load(deps.storage)?;

            to_json_binary(&StateResponse {
                config,
                reward_token: REWARD_TOKEN.load(deps.storage)?,
                last_distributed,
            })
        }
    }
}
