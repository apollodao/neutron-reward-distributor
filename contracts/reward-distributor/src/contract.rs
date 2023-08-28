use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, QueryRequest, Response,
    StdError, StdResult, WasmQuery,
};
use cw20::{Cw20QueryMsg, MinterResponse};
use cw_dex::astroport::AstroportPool;
use cw_vault_standard::{VaultContract, VaultContractUnchecked};
use neutron_astroport_reward_distributor::{
    Config, ConfigUnchecked, ContractError, ExecuteMsg, InstantiateMsg, InternalMsg, QueryMsg,
    StateResponse, CONFIG, LAST_DISTRIBUTED, REWARD_POOL, REWARD_VAULT,
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

    let reward_vault: VaultContract =
        VaultContractUnchecked::new(&msg.reward_vault_addr).check(deps.as_ref())?;

    // Validate reward vault base token as CW20 Astroport LP token
    let reward_lp_token = deps
        .api
        .addr_validate(&reward_vault.base_token)
        .map_err(|_| StdError::generic_err("Invalid base token of reward vault"))?;

    // Query minter of LP token to get reward pool address
    let minter_res: MinterResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: reward_lp_token.to_string(),
        msg: to_binary(&Cw20QueryMsg::Minter {})?,
    }))?;
    let reward_pool_addr = deps.api.addr_validate(&minter_res.minter)?;

    // Query reward pool for pool info to create pool object
    let reward_pool = AstroportPool::new(deps.as_ref(), reward_pool_addr)?;

    // Create config
    let config: Config = ConfigUnchecked {
        distribution_addr: msg.distribution_addr,
        emission_per_second: msg.emission_per_second,
        rewards_start_time: msg.rewards_start_time,
    }
    .check(deps.api)?;

    CONFIG.save(deps.storage, &config)?;
    LAST_DISTRIBUTED.save(deps.storage, &env.block.time.seconds())?;
    REWARD_POOL.save(deps.storage, &reward_pool)?;
    REWARD_VAULT.save(deps.storage, &reward_vault)?;

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
            to_binary(&ownership)
        }
        QueryMsg::State {} => {
            let config = CONFIG.load(deps.storage)?;
            let reward_pool = REWARD_POOL.load(deps.storage)?;
            let reward_vault = REWARD_VAULT.load(deps.storage)?;
            let last_distributed = LAST_DISTRIBUTED.load(deps.storage)?;

            to_binary(&StateResponse {
                config,
                reward_pool,
                reward_vault,
                last_distributed,
            })
        }
    }
}
