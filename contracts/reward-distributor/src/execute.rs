use apollo_cw_asset::{Asset, AssetInfo, AssetList};
use cosmwasm_std::{Deps, DepsMut, Env, Event, MessageInfo, Response, Uint128};
use cw_dex::traits::Pool as PoolTrait;
use neutron_astroport_reward_distributor::{
    ConfigUpdates, ContractError, InternalMsg, CONFIG, LAST_DISTRIBUTED, REWARD_POOL, REWARD_VAULT,
};

pub fn execute_distribute(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let last_distributed = LAST_DISTRIBUTED.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let current_time = env.block.time.seconds();

    // Only distribute if rewards start time has passed
    if current_time < config.rewards_start_time {
        return Err(ContractError::RewardsNotStarted {
            current_time,
            start_time: config.rewards_start_time,
        });
    }

    // Only distribute once per block
    if current_time == last_distributed {
        return Err(ContractError::CanOnlyDistributeOncePerBlock {});
    }

    // Calculate amount of rewards to be distributed
    let time_elapsed = current_time.saturating_sub(last_distributed);
    let redeem_amount = config.emission_per_second * Uint128::from(time_elapsed);

    // Set last distributed time to current time
    LAST_DISTRIBUTED.save(deps.storage, &current_time)?;

    // Only distribute if there are rewards to be distributed
    if redeem_amount.is_zero() {
        return Err(ContractError::NoRewardsToDistribute {});
    }

    // Check contract's balance of vault tokens and error if not enough. This is just so we get a
    // clearer error message rather than the confusing "cannot sub 0 with x".
    let reward_vault = REWARD_VAULT.load(deps.storage)?;
    let vault_token_balance = deps
        .querier
        .query_balance(&env.contract.address, &reward_vault.vault_token)?;
    if vault_token_balance.amount < redeem_amount {
        return Err(ContractError::InsufficientVaultTokenBalance {
            vault_token_balance: vault_token_balance.amount,
            redeem_amount,
        });
    }

    // Redeem rewards from the vault
    let redeem_msg = reward_vault.redeem(redeem_amount, None)?;

    // Create internal callback msg
    let callback_msg = InternalMsg::VaultTokensRedeemed {}.into_cosmos_msg(&env)?;

    let event = Event::new("apollo/neutron-astroport-reward-distributor/execute_distribute")
        .add_attribute("vault_tokens_redeemed", redeem_amount);

    Ok(Response::default()
        .add_message(redeem_msg)
        .add_message(callback_msg)
        .add_event(event))
}

pub fn execute_internal_vault_tokens_redeemed(
    deps: Deps,
    env: Env,
) -> Result<Response, ContractError> {
    let reward_pool = REWARD_POOL.load(deps.storage)?;

    // Query lp token balance
    let reward_lp_token = AssetInfo::Cw20(reward_pool.lp_token_addr.clone());
    let lp_balance = reward_lp_token.query_balance(&deps.querier, env.contract.address.clone())?;
    let lp_tokens = Asset::new(reward_lp_token, lp_balance);

    // Withdraw liquidity with all of contracts LP tokens
    let withdraw_res = reward_pool.withdraw_liquidity(deps, &env, lp_tokens, AssetList::new())?;

    // Create internal callback msg
    let callback_msg = InternalMsg::LpRedeemed {}.into_cosmos_msg(&env)?;

    let event = Event::new(
        "apollo/neutron-astroport-reward-distributor/execute_internal_vault_tokens_redeemed",
    )
    .add_attribute("lp_tokens_redeemed", lp_balance);

    Ok(withdraw_res.add_message(callback_msg).add_event(event))
}

pub fn execute_internal_lp_redeemed(deps: Deps, env: Env) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let reward_pool = REWARD_POOL.load(deps.storage)?;

    // Query contracts balances of pool assets
    let pool_asset_balances: AssetList = AssetList::query_asset_info_balances(
        reward_pool.pool_assets,
        &deps.querier,
        &env.contract.address,
    )?;

    // Create msg to send assets to distribution address
    let send_msgs = pool_asset_balances.transfer_msgs(config.distribution_addr)?;

    let mut event = Event::new(
        "apollo/neutron-astroport-reward-distributor/execute_internal_vault_tokens_redeemed",
    );
    for asset in pool_asset_balances.iter() {
        event = event.add_attribute("asset_distributed", asset.to_string());
    }

    Ok(Response::default().add_messages(send_msgs).add_event(event))
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

    let event = Event::new("apollo/neutron-astroport-reward-distributor/execute_update_config")
        .add_attribute("old_config", format!("{:?}", config))
        .add_attribute("new_config", format!("{:?}", updated_config));

    Ok(Response::default().add_event(event))
}
