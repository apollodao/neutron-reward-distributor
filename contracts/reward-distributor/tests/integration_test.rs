use common::get_test_runner;
use cosmwasm_std::Uint128;
use cw_it::test_tube::Account;
use cw_it::traits::CwItRunner;

use locked_astroport_vault_test_helpers::cw_vault_standard_test_helpers::traits::CwVaultStandardRobot;
use locked_astroport_vault_test_helpers::robot::LockedAstroportVaultRobot;
use neutron_astroport_reward_distributor_test_helpers as test_helpers;

use test_helpers::robot::RewardDistributorRobot;

use crate::common::{DEPS_PATH, UNOPTIMIZED_PATH};

mod common;

#[test]
fn test_initialization() {
    let runner = get_test_runner();
    let admin = RewardDistributorRobot::default_account(&runner);
    let treasury_addr = runner.init_account(&[]).unwrap();
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let robot = RewardDistributorRobot::instantiate(
        &runner,
        &dependencies,
        DEPS_PATH,
        UNOPTIMIZED_PATH,
        treasury_addr.address(),
        &admin,
        1000000u128,
    );

    // Query contract state
    let state = robot.query_state();
    let config = state.config;
    assert_eq!(config.emission_per_second, Uint128::from(1000000u128));
    assert_eq!(config.distribution_addr, robot.distribution_acc.address());
    assert_eq!(state.reward_pool, robot.reward_pool);
    assert_eq!(
        state.reward_pool.lp_token_addr.to_string(),
        robot.reward_vault_robot.base_token()
    );
    assert_eq!(state.reward_vault.addr, robot.reward_vault_robot.vault_addr);
    assert_eq!(
        state.reward_vault.vault_token,
        robot.reward_vault_robot.vault_token()
    );
}

#[test]
fn test_distribute() {
    let runner = get_test_runner();
    let admin = RewardDistributorRobot::default_account(&runner);
    let treasury_addr = runner.init_account(&[]).unwrap();
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let robot = RewardDistributorRobot::instantiate(
        &runner,
        &dependencies,
        DEPS_PATH,
        UNOPTIMIZED_PATH,
        treasury_addr.address(),
        &admin,
        1000000u128,
    );
    let user = robot.reward_vault_robot.new_user(&admin);

    // Query LP token balance
    let base_token_balance = robot
        .reward_vault_robot
        .query_base_token_balance(user.address());
    println!("LP token balance: {}", base_token_balance);

    // Deposit to vault
    robot
        .reward_vault_robot
        .deposit_cw20(base_token_balance, None, &user);

    // Query vault token balance
    let vault_token_balance = robot
        .reward_vault_robot
        .query_vault_token_balance(user.address());

    assert_eq!(vault_token_balance, base_token_balance);
}
