use apollo_cw_asset::{Asset, AssetInfo, AssetList};
use cosmwasm_std::{
    coins, BankMsg, CosmosMsg, Deps, DepsMut, Env, Event, MessageInfo, Response, Uint128,
};
use cw_dex::traits::Pool as PoolTrait;
use neutron_astroport_reward_distributor::{
    ConfigUpdates, ContractError, InternalMsg, RewardType, CONFIG, LAST_DISTRIBUTED, REWARD_TOKEN,
};

pub fn execute_distribute(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let last_distributed = LAST_DISTRIBUTED.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let current_time = env.block.time.seconds();

    // Only distribute if rewards start time has passed
    if current_time < config.rewards_start_time {
        return Ok(Response::new());
    }

    // Only distribute once per second
    if current_time == last_distributed {
        return Ok(Response::new());
    }

    // Calculate amount of rewards to be distributed
    let time_elapsed = current_time.saturating_sub(last_distributed.max(config.rewards_start_time));
    let reward_amount = config.emission_per_second * Uint128::from(time_elapsed);

    let reward_token = REWARD_TOKEN.load(deps.storage)?;

    let mut res = Response::new();

    match reward_token {
        RewardType::Vault { vault, pool: _ } => {
            // Query the vault to see how many base tokens would be returned after
            // redeeming. If zero we return Ok, so that update_config does not fail when
            // trying to distribute.
            let base_token_amount = vault.query_convert_to_assets(&deps.querier, reward_amount)?;
            if base_token_amount.is_zero() {
                return Ok(Response::new());
            }

            // Check contract's balance of vault tokens and error if not enough. This is
            // just so we get a clearer error message rather than the confusing "cannot
            // sub 0 with x".
            let vault_token_balance = deps
                .querier
                .query_balance(&env.contract.address, &vault.vault_token)?;
            if vault_token_balance.amount < reward_amount {
                return Err(ContractError::InsufficientVaultTokenBalance {
                    vault_token_balance: vault_token_balance.amount,
                    redeem_amount: reward_amount,
                });
            }

            // Redeem rewards from the vault
            let redeem_msg = vault.redeem(reward_amount, None)?;

            // Create internal callback msg
            let callback_msg = InternalMsg::VaultTokensRedeemed {}.into_cosmos_msg(&env)?;

            res = res.add_message(redeem_msg).add_message(callback_msg);
        }
        RewardType::LP(pool) => {
            // Create message to withdraw liquidity from pool
            let lp_tokens = Asset::new(AssetInfo::Cw20(pool.lp_token_addr.clone()), reward_amount);
            res = pool.withdraw_liquidity(deps.as_ref(), &env, lp_tokens, AssetList::new())?;

            // Create internal callback msg
            let callback_msg = InternalMsg::LpRedeemed {}.into_cosmos_msg(&env)?;
            res = res.add_message(callback_msg);
        }
        RewardType::Coin(reward_coin_denom) => {
            // Create message to send coins to distribution address
            let send_msg: CosmosMsg = BankMsg::Send {
                to_address: config.distribution_addr.to_string(),
                amount: coins(reward_amount.u128(), reward_coin_denom),
            }
            .into();
            res = res.add_message(send_msg);
        }
    }

    // Set last distributed time to current time
    LAST_DISTRIBUTED.save(deps.storage, &current_time)?;

    let event = Event::new("apollo/neutron-astroport-reward-distributor/execute_distribute")
        .add_attribute("vault_tokens_redeemed", reward_amount);

    Ok(res.add_event(event))
}

pub fn execute_internal_vault_tokens_redeemed(
    deps: Deps,
    env: Env,
) -> Result<Response, ContractError> {
    let reward_pool = REWARD_TOKEN.load(deps.storage)?.into_pool()?;

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
    let reward_pool = REWARD_TOKEN.load(deps.storage)?.into_pool()?;

    // Query contracts balances of pool assets
    let pool_asset_balances: AssetList = AssetList::query_asset_info_balances(
        reward_pool.pool_assets,
        &deps.querier,
        &env.contract.address,
    )?;

    // Create msg to send assets to distribution address
    let send_msgs = pool_asset_balances.transfer_msgs(config.distribution_addr)?;

    let mut event =
        Event::new("apollo/neutron-astroport-reward-distributor/execute_internal_lp_redeemed");
    for asset in pool_asset_balances.iter() {
        event = event.add_attribute("asset_distributed", asset.to_string());
    }

    Ok(Response::default().add_messages(send_msgs).add_event(event))
}

pub fn execute_update_config(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    updates: ConfigUpdates,
) -> Result<Response, ContractError> {
    // only owner can send this message
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let config = CONFIG.load(deps.storage)?;
    let updated_config = config.update(deps.api, updates)?;

    // If we are changing the emission rate or the reward start time, we first need
    // to distribute rewards, so that the emission rate change takes effect from
    // the current block.
    let res = if config.emission_per_second != updated_config.emission_per_second
        || config.rewards_start_time != updated_config.rewards_start_time
    {
        execute_distribute(deps.branch(), env)?
    } else {
        Response::default()
    };

    // Update config
    CONFIG.save(deps.storage, &updated_config)?;

    let event = Event::new("apollo/neutron-astroport-reward-distributor/execute_update_config")
        .add_attribute("old_config", format!("{:?}", config))
        .add_attribute("new_config", format!("{:?}", updated_config));

    Ok(res.add_event(event))
}
