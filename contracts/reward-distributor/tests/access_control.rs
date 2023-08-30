use common::get_test_runner;
use cosmwasm_std::Uint128;
use cw_it::helpers::Unwrap;
use cw_it::robot::TestRobot;
use cw_it::test_tube::Account;
use cw_it::traits::CwItRunner;

use locked_astroport_vault_test_helpers::robot::LockedAstroportVaultRobot;
use neutron_astroport_reward_distributor::{ConfigUpdates, ExecuteMsg, InternalMsg};
use neutron_astroport_reward_distributor_test_helpers as test_helpers;

use test_helpers::robot::RewardDistributorRobot;

use crate::common::{DEPS_PATH, UNOPTIMIZED_PATH};

pub mod common;

#[test]
fn update_ownership_can_only_be_called_by_admin() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = RewardDistributorRobot::default_account(&runner);
    let treasury_addr = runner.init_account(&[]).unwrap();
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let emission_per_second = 100u128;
    let rewards_start_time = runner.query_block_time_nanos() / 1_000_000_000;
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

    let user = runner.init_default_account().unwrap();
    let action = cw_ownable::Action::TransferOwnership {
        new_owner: admin.address(),
        expiry: None,
    };

    // Try calling update_ownership as non-admin, should fail. Then try calling as
    // admin, should work.
    robot
        .update_ownership(
            action.clone(),
            Unwrap::Err("Caller is not the contract's current owner"),
            &user,
        )
        .update_ownership(action, Unwrap::Ok, &admin);
}

#[test]
fn update_config_can_only_be_called_by_admin() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = RewardDistributorRobot::default_account(&runner);
    let treasury_addr = runner.init_account(&[]).unwrap();
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let emission_per_second = 100u128;
    let rewards_start_time = runner.query_block_time_nanos() / 1_000_000_000;
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

    let user = runner.init_default_account().unwrap();

    let mut config_updates = ConfigUpdates::default();
    config_updates.emission_per_second(Uint128::new(420));

    // Try calling update_config as non-admin, should fail. Then try calling as
    // admin, should work.
    robot
        .update_config(
            config_updates.clone(),
            Unwrap::Err("Caller is not the contract's current owner"),
            &user,
        )
        .update_config(config_updates, Unwrap::Ok, &admin);
}

#[test]
fn internal_msg_can_only_be_called_by_contract() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = RewardDistributorRobot::default_account(&runner);
    let treasury_addr = runner.init_account(&[]).unwrap();
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let emission_per_second = 100u128;
    let rewards_start_time = runner.query_block_time_nanos() / 1_000_000_000;
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

    let err = robot
        .wasm()
        .execute(
            &robot.reward_distributor_addr,
            &ExecuteMsg::Internal(InternalMsg::LpRedeemed {}),
            &[],
            &admin,
        )
        .unwrap_err();
    assert!(err.to_string().contains("Unauthorized"));

    let err = robot
        .wasm()
        .execute(
            &robot.reward_distributor_addr,
            &ExecuteMsg::Internal(InternalMsg::VaultTokensRedeemed {}),
            &[],
            &admin,
        )
        .unwrap_err();
    assert!(err.to_string().contains("Unauthorized"));
}
