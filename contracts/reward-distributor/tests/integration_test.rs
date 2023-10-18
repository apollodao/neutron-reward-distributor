use common::get_test_runner;
use cosmwasm_std::{coin, Uint128};
use cw_it::helpers::Unwrap;
use cw_it::test_tube::Account;
use cw_it::traits::CwItRunner;
use locked_astroport_vault::helpers::INITIAL_VAULT_TOKENS_PER_BASE_TOKEN;
use locked_astroport_vault_test_helpers::cw_vault_standard_test_helpers::traits::CwVaultStandardRobot;
use locked_astroport_vault_test_helpers::robot::LockedAstroportVaultRobot;
use neutron_astroport_reward_distributor::RewardType;
use neutron_astroport_reward_distributor_test_helpers as test_helpers;

use test_helpers::robot::RewardDistributorRobot;

use crate::common::{DEPS_PATH, UNOPTIMIZED_PATH};

mod common;

#[test]
fn test_initialization() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = RewardDistributorRobot::default_account(&runner);
    let treasury_addr = runner.init_account(&[]).unwrap();
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let rewards_start_time = runner.query_block_time_nanos() / 1_000_000_000;
    let robot = RewardDistributorRobot::instantiate(
        &runner,
        &dependencies,
        DEPS_PATH,
        UNOPTIMIZED_PATH,
        treasury_addr.address(),
        &admin,
        1000000u128,
        rewards_start_time,
    );

    // Query contract state
    let state = robot.query_state();
    let config = state.config;
    assert_eq!(config.emission_per_second, Uint128::from(1000000u128));
    assert_eq!(config.distribution_addr, robot.distribution_acc.address());
    assert!(matches!(
        state.reward_token,
        RewardType::Vault { vault, pool } if vault.addr == robot.reward_vault_robot.vault_addr && pool.lp_token_addr == robot.reward_vault_robot.base_token()
    ));

    // Query ownership
    let ownership = robot.query_ownership();
    assert_eq!(ownership.owner.unwrap(), admin.address());
}

#[test]
fn distribute_errors_when_not_enough_vault_tokens_in_contract() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = RewardDistributorRobot::default_account(&runner);
    let treasury_addr = runner.init_account(&[]).unwrap();
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let emission_per_second = 100_000_000u128;
    let rewards_start_time = runner.query_block_time_nanos() / 1_000_000_000 + 5;
    let robot = RewardDistributorRobot::instantiate(
        &runner,
        &dependencies,
        DEPS_PATH,
        UNOPTIMIZED_PATH,
        treasury_addr.address(),
        &admin,
        emission_per_second,
        rewards_start_time,
    );

    // Increase time to start rewards
    robot.increase_time(6);

    // Try to distribute rewards, should fail
    robot.distribute(Unwrap::Err("Insufficient vault token balance"), &admin);

    // Deposit to vault and donate vault tokens to reward distributor and try to
    // distribute again. Should work.
    let deposit_amount = Uint128::new(10000000u128);
    robot
        .deposit_to_distributor(deposit_amount, Unwrap::Ok, &admin)
        .distribute(Unwrap::Ok, &admin);
}

#[test]
fn test_correct_distribute() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = RewardDistributorRobot::default_account(&runner);
    let treasury_addr = runner.init_account(&[]).unwrap();
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let emission_per_second = 100_000u128;
    let init_time = runner.query_block_time_nanos() / 1_000_000_000;
    let rewards_start_time = init_time + 5;
    let robot = RewardDistributorRobot::instantiate(
        &runner,
        &dependencies,
        DEPS_PATH,
        UNOPTIMIZED_PATH,
        treasury_addr.address(),
        &admin,
        emission_per_second,
        rewards_start_time,
    );

    let vault_robot = &robot.reward_vault_robot;

    // Deposit to vault and send vault tokens to reward distributor
    let base_token_balance = vault_robot.query_base_token_balance(admin.address());
    let deposit_amount = base_token_balance / Uint128::new(10);
    let vault_token_balance = robot
        .deposit_to_distributor(deposit_amount, Unwrap::Ok, &admin)
        .reward_vault_robot
        .query_vault_token_balance(&robot.reward_distributor_addr);

    // Distribute rewards and check balances
    let time_elapsed = 1000u64;
    robot
        .assert_distribution_acc_balances_eq(&[])
        .distribute(Unwrap::Ok, &admin)
        .increase_time(5) // Rewards have started
        .assert_distribution_acc_balances_eq(&[])
        .increase_time(time_elapsed)
        .distribute(Unwrap::Ok, &admin)
        .assert_distribution_acc_balances_eq(&[
            coin(
                (emission_per_second * time_elapsed as u128)
                    / INITIAL_VAULT_TOKENS_PER_BASE_TOKEN.u128(),
                "uaxl",
            ),
            coin(
                (emission_per_second * time_elapsed as u128)
                    / INITIAL_VAULT_TOKENS_PER_BASE_TOKEN.u128(),
                "untrn",
            ),
        ]);

    // Vault token balance of reward distributor should have decreased with the
    // amount distributed
    vault_robot.assert_vault_token_balance_eq(
        robot.reward_distributor_addr,
        vault_token_balance.u128() - emission_per_second * time_elapsed as u128,
    );
}

#[test]
fn distribute_does_not_error_when_distributed_vault_token_amount_would_give_zero_base_tokens() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = RewardDistributorRobot::default_account(&runner);
    let treasury_addr = runner.init_account(&[]).unwrap();
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let emission_per_second = 100u128;
    let init_time = runner.query_block_time_nanos() / 1_000_000_000;
    let rewards_start_time = init_time + 5;
    let robot = RewardDistributorRobot::instantiate(
        &runner,
        &dependencies,
        DEPS_PATH,
        UNOPTIMIZED_PATH,
        treasury_addr.address(),
        &admin,
        emission_per_second,
        rewards_start_time,
    );

    let vault_robot = &robot.reward_vault_robot;

    // Deposit to vault and send vault tokens to reward distributor
    let deposit_amount = Uint128::new(100);
    let vault_token_balance = robot
        .deposit_to_distributor(deposit_amount, Unwrap::Ok, &admin)
        .reward_vault_robot
        .query_vault_token_balance(&robot.reward_distributor_addr);
    assert_eq!(
        vault_token_balance,
        deposit_amount * INITIAL_VAULT_TOKENS_PER_BASE_TOKEN // 100 * 1_000_000 = 100_000_000
    );

    // Distribute rewards and check balances.
    // After time_elapsed the amount available to distribute would be 10 * 100 =
    // 1000 vault tokens, which would give 0 base tokens.
    let time_elapsed = 10u64;
    robot
        .assert_distribution_acc_balances_eq(&[])
        .distribute(Unwrap::Ok, &admin)
        .increase_time(5) // Rewards have started
        .assert_distribution_acc_balances_eq(&[])
        .increase_time(time_elapsed)
        .distribute(Unwrap::Ok, &admin)
        .assert_distribution_acc_balances_eq(&[]);

    // Vault token balance of reward distributor should have remained the same, as
    // no rewards were distributed
    vault_robot.assert_vault_token_balance_eq(robot.reward_distributor_addr, vault_token_balance);
}
