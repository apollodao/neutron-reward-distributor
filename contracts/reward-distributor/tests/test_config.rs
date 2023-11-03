use common::get_test_runner;
use cosmwasm_std::{coin, Addr, Uint128};
use cw_it::helpers::Unwrap;
use cw_it::test_tube::Account;
use cw_it::traits::CwItRunner;

use locked_astroport_vault_test_helpers::cw_vault_standard_test_helpers::traits::CwVaultStandardRobot;
use locked_astroport_vault_test_helpers::robot::LockedAstroportVaultRobot;
use neutron_astroport_reward_distributor::{Config, ConfigUpdates};
use neutron_astroport_reward_distributor_test_helpers as test_helpers;

use test_helpers::robot::{RewardDistributorRobot, TestRewardType};

use crate::common::{DEPS_PATH, UNOPTIMIZED_PATH};

mod common;

#[test]
fn update_config_works_correctly() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = RewardDistributorRobot::default_account(&runner);
    let treasury_addr = runner.init_account(&[]).unwrap();
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let emission_per_second = 100_000_000u128;
    let init_time = runner.query_block_time_nanos() / 1_000_000_000;
    let rewards_start_time = init_time + 5;
    let robot = RewardDistributorRobot::instantiate(
        &runner,
        &dependencies,
        DEPS_PATH,
        UNOPTIMIZED_PATH,
        treasury_addr.address(),
        TestRewardType::VaultToken,
        &admin,
        emission_per_second,
        rewards_start_time,
    );

    let mut config_updates = ConfigUpdates::default();
    let new_distr_addr = runner.init_account(&[]).unwrap().address();
    config_updates.distribution_addr(new_distr_addr.clone());
    config_updates.emission_per_second(Uint128::new(420_000_000));
    config_updates.rewards_start_time(rewards_start_time + 100);
    robot
        .update_config(config_updates, Unwrap::Ok, &admin)
        .assert_config_eq(&Config {
            emission_per_second: Uint128::new(420_000_000),
            distribution_addr: Addr::unchecked(new_distr_addr),
            rewards_start_time: rewards_start_time + 100,
        });
}

#[test]
/// Ensures that pending rewards are distributed when emission rate or start
/// time changes
fn update_config_distributes_rewards_if_emission_rate_or_start_time_changes() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = RewardDistributorRobot::default_account(&runner);
    let treasury_addr = runner.init_account(&[]).unwrap();
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let emission_per_second = 100_000_000u128;
    let init_time = runner.query_block_time_nanos() / 1_000_000_000;
    let rewards_start_time = init_time + 5;
    let robot = RewardDistributorRobot::instantiate(
        &runner,
        &dependencies,
        DEPS_PATH,
        UNOPTIMIZED_PATH,
        treasury_addr.address(),
        TestRewardType::VaultToken,
        &admin,
        emission_per_second,
        rewards_start_time,
    );

    let vault_robot = &robot.reward_vault_robot;

    // Deposit to vault and send vault tokens to reward distributor
    let base_token_balance = vault_robot.query_base_token_balance(admin.address());
    robot.deposit_to_distributor(base_token_balance, Unwrap::Ok, &admin);

    // Update emission rate and check that rewards are distributed
    let time_elapsed = 100u64;
    let mut config_updates = ConfigUpdates::default();
    config_updates.emission_per_second(Uint128::new(420_000_000));
    let balances_after_distribution = robot
        .increase_time(time_elapsed)
        .assert_distribution_acc_balances_eq(&[])
        .update_config(config_updates, Unwrap::Ok, &admin)
        .assert_distribution_acc_balances_gt(&[coin(0, "uaxl"), coin(0, "untrn")])
        .query_distribution_acc_balances();

    // Update start time and check that rewards are distributed
    let mut config_updates = ConfigUpdates::default();
    config_updates.rewards_start_time(rewards_start_time + time_elapsed * 2 + 10);
    robot
        .increase_time(time_elapsed)
        .update_config(config_updates, Unwrap::Ok, &admin)
        .assert_distribution_acc_balances_gt(&balances_after_distribution);
}

#[test]
fn update_config_does_not_distribute_rewards_when_not_changing_emission_rate_or_start_time() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = RewardDistributorRobot::default_account(&runner);
    let treasury_addr = runner.init_account(&[]).unwrap();
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let emission_per_second = 100_000_000u128;
    let init_time = runner.query_block_time_nanos() / 1_000_000_000;
    let rewards_start_time = init_time + 5;
    let robot = RewardDistributorRobot::instantiate(
        &runner,
        &dependencies,
        DEPS_PATH,
        UNOPTIMIZED_PATH,
        treasury_addr.address(),
        TestRewardType::VaultToken,
        &admin,
        emission_per_second,
        rewards_start_time,
    );

    let mut config_updates = ConfigUpdates::default();
    let new_distr_addr = runner.init_account(&[]).unwrap().address();
    config_updates.distribution_addr(new_distr_addr.clone());
    robot
        .update_config(config_updates, Unwrap::Ok, &admin)
        .assert_config_eq(&Config {
            emission_per_second: emission_per_second.into(),
            distribution_addr: Addr::unchecked(new_distr_addr),
            rewards_start_time,
        })
        .assert_distribution_acc_balances_eq(&[]);
}
