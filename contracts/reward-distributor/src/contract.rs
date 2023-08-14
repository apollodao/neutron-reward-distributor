use apollo_cw_asset::AssetInfo;
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw_vault_standard::VaultContractUnchecked;
use reward_distributor::{
    Config, ConfigResponse, ContractError, ExecuteMsg, InstantiateMsg, InternalMsg, QueryMsg,
    CONFIG, LAST_DISTRIBUTED,
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

    let reward_vault = VaultContractUnchecked::new(&msg.reward_vault_addr).check(deps.api)?;

    // Query vault for vault token and lp token denom
    let vault_info = reward_vault.query_vault_info(&deps.querier)?;
    let reward_lp_token = match deps.api.addr_validate(&vault_info.base_token) {
        Ok(addr) => AssetInfo::Cw20(addr),
        Err(_) => AssetInfo::Native(vault_info.base_token),
    };

    let config = Config {
        distribution_addr: deps.api.addr_validate(&msg.distribution_addr)?,
        reward_vault,
        emission_per_second: msg.emission_per_second,
        reward_lp_token,
        reward_vt_denom: vault_info.vault_token,
        reward_pool: msg.reward_pool,
    };

    CONFIG.save(deps.storage, &config)?;
    LAST_DISTRIBUTED.save(deps.storage, &env.block.time.seconds())?;

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
            cw_ownable::update_ownership(deps, &env.block, &info.sender, action)?;
            Ok(Response::default())
        }
        ExecuteMsg::Distribute {} => execute::execute_distribute(deps, env),
        ExecuteMsg::Internal(msg) => match msg {
            InternalMsg::VaultTokensRedeemed {} => {
                execute::execute_internal_vault_tokens_redeemed(deps.as_ref(), env)
            }
            InternalMsg::LpRedeemed {} => execute::execute_internal_lp_redeemed(deps.as_ref(), env),
        },
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => {
            let ownership = cw_ownable::get_ownership(deps.storage)?;
            to_binary(&ownership)
        }
        QueryMsg::Config {} => {
            let config = CONFIG.load(deps.storage)?;
            to_binary(&ConfigResponse {
                distribution_addr: config.distribution_addr.to_string(),
                reward_vault_addr: config.reward_vault.addr.to_string(),
                emission_per_second: config.emission_per_second,
                reward_lp_token: config.reward_lp_token,
                reward_pool: config.reward_pool,
                reward_vt_denom: config.reward_vt_denom,
            })
        }
    }
}
