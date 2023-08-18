use common::get_test_runner;
use cosmwasm_std::{coin, Uint128};
use cw_it::robot::TestRobot;
use cw_it::test_tube::Account;
use cw_it::traits::CwItRunner;

use locked_astroport_vault_test_helpers::robot::LockedAstroportVaultRobot;
use locked_astroport_vault_test_helpers::{
    cw_vault_standard_test_helpers::traits::CwVaultStandardRobot, helpers::Unwrap,
};
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
fn distribute_errors_on_no_rewards_to_distribute() {
    let runner = get_test_runner();
    let admin = RewardDistributorRobot::default_account(&runner);
    let treasury_addr = runner.init_account(&[]).unwrap();
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let emission_per_second = 0u128;
    let robot = RewardDistributorRobot::instantiate(
        &runner,
        &dependencies,
        DEPS_PATH,
        UNOPTIMIZED_PATH,
        treasury_addr.address(),
        &admin,
        emission_per_second,
    );

    robot
        .distribute(Unwrap::Err("Can only distribute once per block"), &admin)
        .increase_time(5)
        .distribute(Unwrap::Err("No rewards to distribute"), &admin);
}

#[test]
fn test_distribute() {
    let runner = get_test_runner();
    let admin = RewardDistributorRobot::default_account(&runner);
    let treasury_addr = runner.init_account(&[]).unwrap();
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let emission_per_second = 100u128;
    let robot = RewardDistributorRobot::instantiate(
        &runner,
        &dependencies,
        DEPS_PATH,
        UNOPTIMIZED_PATH,
        treasury_addr.address(),
        &admin,
        emission_per_second,
    );

    let vault_robot = &robot.reward_vault_robot;

    // Deposit to vault and send vault tokens to reward distributor
    let base_token_balance = vault_robot.query_base_token_balance(admin.address());
    let deposit_amount = base_token_balance / Uint128::new(10);
    println!("deposit_amount: {}", deposit_amount);
    vault_robot
        .deposit_cw20(deposit_amount, None, &admin)
        .assert_vault_token_balance_eq(admin.address(), deposit_amount)
        .send_native_tokens(
            &admin,
            &robot.reward_distributor_addr,
            deposit_amount,
            &vault_robot.vault_token(),
        );

    // Distribute rewards and check balances
    let time_elapsed = 1000u64;
    robot
        .assert_distribution_acc_balances_eq(&[])
        .distribute(Unwrap::Err("Can only distribute once per block"), &admin)
        .assert_distribution_acc_balances_eq(&[])
        .increase_time(time_elapsed)
        .distribute(Unwrap::Ok, &admin)
        .assert_distribution_acc_balances_eq(&[
            coin(emission_per_second * time_elapsed as u128 - 1, "uaxl"),
            coin(emission_per_second * time_elapsed as u128 - 1, "untrn"),
        ]);

    // Vault token balance of reward distributor should have decreased with the amount distributed
    vault_robot.assert_vault_token_balance_eq(
        robot.reward_distributor_addr,
        deposit_amount.u128() - emission_per_second * time_elapsed as u128,
    );
}
