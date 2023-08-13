use std::str::FromStr;

use apollo_cw_asset::Asset;
use common::{get_test_runner, DefaultRewardDistributorRobot};
use cosmwasm_std::{Coins, Uint128};
use cw_dex::traits::Pool;
use cw_it::astroport::astroport::asset::Asset as AstroAsset;
use cw_it::{astroport::robot::AstroportTestRobot, test_tube::Account};

use crate::common::{RewardDistributorRobot, DEFAULT_LIQ};

mod common;

#[test]
fn test_initialization() {
    let runner = get_test_runner();
    let admin = DefaultRewardDistributorRobot::default_account(&runner);
    let robot = DefaultRewardDistributorRobot::instantiate(&runner, &admin, 1000000u128);

    // Query config
    let config = robot.query_config();
    assert_eq!(config.emission_per_second, Uint128::from(1000000u128));
    assert_eq!(config.distribution_addr, robot.distribution_acc.address());
    assert_eq!(
        config.reward_lp_token.to_string(),
        robot.reward_vault_robot.base_token
    );
    assert_eq!(
        config.reward_vault_addr,
        robot.reward_vault_robot.vault_addr
    );
    assert_eq!(config.reward_vt_denom, robot.reward_vault_robot.vault_token);
    assert_eq!(
        config.reward_pool.lp_token().to_string(),
        robot.reward_lp_addr
    )
}

#[test]
fn test_distribute() {
    let runner = get_test_runner();
    let admin = DefaultRewardDistributorRobot::default_account(&runner);
    let robot = DefaultRewardDistributorRobot::instantiate(&runner, &admin, 1000000u128);

    // Provide liquidity to reward pool
    let assets: Vec<AstroAsset> = Coins::from_str(DEFAULT_LIQ)
        .unwrap()
        .into_vec()
        .into_iter()
        .map(|x| Asset::from(x).into())
        .collect();
    robot.provide_liquidity(&robot.reward_pool_addr, assets, &admin);

    // Query LP token balance
    let lp_token_balance = robot.query_cw20_balance(&robot.reward_lp_addr, &admin.address());
    println!("LP token balance: {}", lp_token_balance);

    // Deposit to vault
    robot
        .increase_cw20_allowance(
            &robot.reward_lp_addr,
            &robot.reward_vault_robot.vault_addr,
            lp_token_balance,
            &admin,
        )
        .reward_vault_robot
        .deposit_cw20_to_vault(lp_token_balance, &admin);
    // Query vault token balance
    let vault_token_balance = robot
        .reward_vault_robot
        .query_vault_token_balance(admin.address());
    assert_eq!(vault_token_balance, Uint128::from(lp_token_balance));
}
