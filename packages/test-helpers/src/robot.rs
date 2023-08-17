use std::str::FromStr;

use cosmwasm_std::{Coin, Coins, Uint128};
use cw_dex::astroport::AstroportPool;
use cw_it::astroport::robot::AstroportTestRobot;
use cw_it::astroport::utils::AstroportContracts;
use cw_it::cw_multi_test::ContractWrapper;
use cw_it::osmosis_std::types::cosmos::bank::v1beta1::QueryAllBalancesRequest;
use cw_it::robot::TestRobot;
use cw_it::test_tube::{Account, Module, SigningAccount, Wasm};
use cw_it::traits::CwItRunner;
use cw_it::{ContractType, TestRunner};
use locked_astroport_vault_test_helpers::robot::{
    LockedAstroportVaultRobot, LockedVaultDependencies,
};
use neutron_astroport_reward_distributor as reward_distributor;
use neutron_astroport_reward_distributor::InstantiateMsg;

#[cfg(feature = "osmosis-test-tube")]
use cw_it::Artifact;

pub const REWARD_DISTRIBUTOR_WASM_NAME: &str = "neutron_astroport_reward_distributor_contract.wasm";

/// The fee you need to pay to create a new denom with Token Factory.
pub const DENOM_CREATION_FEE: &str = "10000000uosmo";

/// The default coins to fund new accounts with
pub const DEFAULT_COINS: &str =
    "1000000000000000000uosmo,1000000000000000000untrn,1000000000000000000uaxl,1000000000000000000uastro";

/// The default liquidity for the reward pool
pub const DEFAULT_LIQ: &str = "1000000000untrn,1000000000uaxl"; // 1K NTRN, 1K AXL

/// A helper struct implementing the Robot testing pattern for testing the
/// reward distributor contract.
pub struct RewardDistributorRobot<'a> {
    pub runner: &'a TestRunner<'a>,
    pub astroport_contracts: AstroportContracts,
    pub reward_distributor_addr: String,
    pub distribution_acc: SigningAccount,
    pub reward_pool: AstroportPool,
    pub reward_vault_robot: LockedAstroportVaultRobot<'a>,
}

/// A trait with helper functions for testing the reward distributor contract.
impl<'a> RewardDistributorRobot<'a> {
    /// Returns the contract code to be able to upload the contract
    pub fn contract(runner: &TestRunner, _artifacts_dir: &str) -> ContractType {
        match runner {
            TestRunner::MultiTest(_) => {
                ContractType::MultiTestContract(Box::new(ContractWrapper::new_with_empty(
                    neutron_astroport_reward_distributor_contract::contract::execute,
                    neutron_astroport_reward_distributor_contract::contract::instantiate,
                    neutron_astroport_reward_distributor_contract::contract::query,
                )))
            }
            #[cfg(feature = "osmosis-test-tube")]
            TestRunner::OsmosisTestApp(_) => {
                let path = format!("{}/{}", _artifacts_dir, REWARD_DISTRIBUTOR_WASM_NAME);
                println!("Loading contract from {}", path);
                ContractType::Artifact(Artifact::Local(path))
            }
            _ => panic!("Unsupported test runner"),
        }
    }

    // Creates a new account with default coins
    pub fn default_account(runner: &'a TestRunner) -> SigningAccount {
        runner
            .init_account(&Coins::from_str(DEFAULT_COINS).unwrap().into_vec())
            .unwrap()
    }

    /// Uploads and instantiates the reward distributor contract as well as all
    /// dependencies. Returns an instance of the default testing robot.
    pub fn instantiate(
        runner: &'a TestRunner,
        vault_dependencies: &'a LockedVaultDependencies<'a>,
        dependency_artifacts_dir: &str,
        artifacts_dir: &str,
        vault_treasury_addr: String,
        admin: &'a SigningAccount,
        emission_per_second: impl Into<Uint128>,
    ) -> Self {
        // Create vault for reward pool
        let (reward_vault_robot, axl_ntrn_pool, _astro_ntrn_pool) =
            LockedAstroportVaultRobot::new_unlocked_axlr_ntrn_vault(
                runner,
                LockedAstroportVaultRobot::contract(runner, dependency_artifacts_dir),
                Coin::from_str(DENOM_CREATION_FEE).unwrap(),
                vault_treasury_addr,
                vault_dependencies,
                admin,
            );

        // Upload and instantiate reward distributor contract
        let code = Self::contract(runner, artifacts_dir);
        let code_id = runner.store_code(code, admin).unwrap();
        let distribution_acc = runner.init_account(&[]).unwrap();
        let msg = InstantiateMsg {
            distribution_addr: distribution_acc.address(),
            emission_per_second: emission_per_second.into(),
            owner: admin.address(),
            reward_vault_addr: reward_vault_robot.vault_addr.clone(),
        };
        let contract_addr = Wasm::new(runner)
            .instantiate(code_id, &msg, Some(&admin.address()), None, &[], admin)
            .unwrap()
            .data
            .address;

        Self {
            runner,
            astroport_contracts: vault_dependencies.astroport_contracts.clone(),
            reward_distributor_addr: contract_addr,
            distribution_acc,
            reward_pool: axl_ntrn_pool,
            reward_vault_robot,
        }
    }

    pub fn distribute(&self, signer: &SigningAccount) -> &Self {
        let msg = reward_distributor::msg::ExecuteMsg::Distribute {};
        self.wasm()
            .execute(&self.reward_distributor_addr, &msg, &[], signer)
            .unwrap();
        self
    }

    pub fn increase_time(&self, seconds: u64) -> &Self {
        self.runner.increase_time(seconds).unwrap();
        self
    }

    pub fn query_state(&self) -> reward_distributor::msg::StateResponse {
        let query_msg = reward_distributor::msg::QueryMsg::State {};
        self.wasm()
            .query(&self.reward_distributor_addr, &query_msg)
            .unwrap()
    }

    pub fn query_distribution_acc_balances(&self) -> Vec<Coin> {
        // self.query_balances(&self.distribution_acc.address())
        self.bank()
            .query_all_balances(&QueryAllBalancesRequest {
                address: self.distribution_acc.address(),
                ..Default::default()
            })
            .unwrap()
            .balances
            .into_iter()
            .map(|b| Coin {
                denom: b.denom,
                amount: Uint128::from_str(&b.amount).unwrap(),
            })
            .collect()
    }

    pub fn assert_distribution_acc_balances_eq(&self, expected: &[Coin]) -> &Self {
        assert_eq!(
            self.query_distribution_acc_balances(),
            expected,
            "Distribution account balances do not match"
        );
        self
    }

    pub fn assert_distribution_acc_balances_gt(&self, expected: &[Coin]) -> &Self {
        let actual = self.query_distribution_acc_balances();
        for (i, coin) in expected.iter().enumerate() {
            assert!(
                actual[i].amount > coin.amount,
                "Distribution account balance {} is not greater than {}",
                actual[i].amount,
                coin.amount
            );
        }
        self
    }
}

impl<'a> TestRobot<'a, TestRunner<'a>> for RewardDistributorRobot<'a> {
    fn runner(&self) -> &'a TestRunner<'a> {
        self.runner
    }
}

impl<'a> AstroportTestRobot<'a, TestRunner<'a>> for RewardDistributorRobot<'a> {
    fn astroport_contracts(&self) -> &AstroportContracts {
        &self.astroport_contracts
    }
}
