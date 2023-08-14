use apollo_cw_asset::{Asset, AssetList};
use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128};
use cw_dex::traits::Pool as PoolTrait;
use reward_distributor::{ConfigUpdates, ContractError, InternalMsg, CONFIG, LAST_DISTRIBUTED};

pub fn execute_distribute(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let last_distributed = LAST_DISTRIBUTED.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let current_time = env.block.time.seconds();

    // Only distribute once per block
    if current_time == last_distributed {
        return Ok(Response::default());
    }

    // Calculate amount of rewards to be distributed
    let time_elapsed = current_time - last_distributed;
    let redeem_amount = config.emission_per_second * Uint128::from(time_elapsed);

    // Set last distributed time to current time
    LAST_DISTRIBUTED.save(deps.storage, &current_time)?;

    // Only distribute if there are rewards to be distributed
    if redeem_amount.is_zero() {
        return Ok(Response::default());
    }

    // Redeem rewards from the vault
    let redeem_msg = config
        .reward_vault
        .redeem(redeem_amount, &config.reward_vt_denom, None)?;

    // Create internal callback msg
    let callback_msg = InternalMsg::VaultTokensRedeemed {}.into_cosmos_msg(&env)?;

    Ok(Response::default()
        .add_message(redeem_msg)
        .add_message(callback_msg))
}

pub fn execute_internal_vault_tokens_redeemed(
    deps: Deps,
    env: Env,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Query lp token balance
    let lp_balance = config
        .reward_lp_token
        .query_balance(&deps.querier, env.contract.address.clone())?;
    let lp_tokens = Asset::new(config.reward_lp_token, lp_balance);

    // Withdraw liquidity with all of contracts LP tokens
    let withdraw_res =
        config
            .reward_pool
            .withdraw_liquidity(deps, &env, lp_tokens, AssetList::new())?;

    // Create internal callback msg
    let callback_msg = InternalMsg::LpRedeemed {}.into_cosmos_msg(&env)?;

    Ok(withdraw_res.add_message(callback_msg))
}

pub fn execute_internal_lp_redeemed(deps: Deps, env: Env) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Query contracts balances of pool assets
    let pool_asset_infos = config.reward_pool.pool_assets(deps)?;
    let pool_asset_balances: AssetList = pool_asset_infos
        .into_iter()
        .map(|x| {
            Ok(Asset::new(
                x.clone(),
                x.query_balance(&deps.querier, env.contract.address.clone())?,
            ))
        })
        .collect::<StdResult<Vec<_>>>()?
        .into();

    // Create msg to send assets to distribution address
    let send_msgs = pool_asset_balances.transfer_msgs(config.distribution_addr)?;

    Ok(Response::default().add_messages(send_msgs))
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    updates: ConfigUpdates,
) -> Result<Response, ContractError> {
    // only owner can send this message
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let config = CONFIG.load(deps.storage)?;
    let updated_config = config.update(deps.api, updates)?;

    // Update config
    CONFIG.save(deps.storage, &updated_config)?;

    Ok(Response::default())
}
