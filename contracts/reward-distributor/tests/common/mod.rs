use std::str::FromStr;

use apollo_cw_asset::AssetInfo;
use cosmwasm_std::{Addr, Coin, Coins, Uint128};
use cw_dex::astroport::AstroportPool;
use cw_it::{
    astroport::{
        astroport::{asset::AssetInfo as AstroAssetInfo, factory::PairType},
        robot::{AstroportTestRobot, DefaultAstroportRobot},
        utils::AstroportContracts,
    },
    cw_multi_test::{ContractWrapper, StargateKeeper, StargateMessageHandler},
    multi_test::{modules::TokenFactory, MultiTestRunner},
    robot::TestRobot,
    test_tube::{Account, Module, SigningAccount, Wasm},
    traits::CwItRunner,
    Artifact, ContractType, TestRunner,
};
use reward_distributor::InstantiateMsg;

/// The fee you need to pay to create a new denom with Token Factory.
pub const DENOM_CREATION_FEE: &str = "10000000untrn";

/// The cw-multi-test mock of Token Factory.
pub const TOKEN_FACTORY: &TokenFactory =
    &TokenFactory::new("factory", 32, 16, 59 + 16, DENOM_CREATION_FEE);

/// The default coins to fund new accounts with
pub const DEFAULT_COINS: &str = "1000000000000000000untrn,1000000000000000000uaxlr";

/// The default liquidity for the reward pool
pub const DEFAULT_LIQ: &str = "1000000000untrn,1000000000uaxlr"; // 1K NTRN, 1K AXLR

/// Helper function to get a test runner based on env var `TEST_RUNNER_TYPE`.
pub fn get_test_runner<'a>() -> TestRunner<'a> {
    match option_env!("TEST_RUNNER_TYPE").unwrap_or("multi-test") {
        "multi-test" => {
            let mut stargate_keeper = StargateKeeper::new();
            TOKEN_FACTORY.register_msgs(&mut stargate_keeper);

            TestRunner::MultiTest(MultiTestRunner::new_with_stargate("osmo", stargate_keeper))
        }
        "osmosis-test-app" => {
            TestRunner::OsmosisTestApp(cw_it::osmosis_test_tube::OsmosisTestApp::new())
        }
        _ => panic!("Unsupported test runner type"),
    }
}

/// A trait with helper functions for testing the reward distributor contract.
pub trait RewardDistributorRobot<'a>:
    TestRobot<'a, TestRunner<'a>> + AstroportTestRobot<'a, TestRunner<'a>>
{
    /// Creates a new instance of the default reward distributor robot.
    fn default_reward_distributor_robot(
        runner: &'a TestRunner,
        astroport_contracts: AstroportContracts,
        reward_distributor_addr: String,
        distribution_acc: SigningAccount,
        reward_pool_addr: String,
        reward_lp_addr: String,
        reward_vault_robot: DefaultVaultRobot<'a, TestRunner<'a>>,
    ) -> DefaultRewardDistributorRobot<'a> {
        DefaultRewardDistributorRobot {
            runner,
            astroport_contracts,
            reward_distributor_addr,
            distribution_acc,
            reward_pool_addr,
            reward_lp_addr,
            reward_vault_robot,
        }
    }

    /// Returns the address of the reward distributor contract.
    fn reward_distributor_addr(&self) -> &str;

    /// Returns the contract code to be able to upload the contract
    fn contract(runner: &TestRunner) -> ContractType {
        match runner {
            TestRunner::MultiTest(_) => {
                ContractType::MultiTestContract(Box::new(ContractWrapper::new_with_empty(
                    reward_distributor_contract::contract::execute,
                    reward_distributor_contract::contract::instantiate,
                    reward_distributor_contract::contract::query,
                )))
            }
            TestRunner::OsmosisTestApp(_) => {
                ContractType::Artifact(Artifact::Local("".to_string()))
            }
            _ => panic!("Unsupported test runner"),
        }
    }

    // Creates a new account with default coins
    fn default_account(runner: &'a TestRunner) -> SigningAccount {
        runner
            .init_account(&Coins::from_str(DEFAULT_COINS).unwrap().into_vec())
            .unwrap()
    }

    /// Uploads and instantiates the reward distributor contract as well as all dependencies.
    /// Returns an instance of the default testing robot.
    fn instantiate(
        runner: &'a TestRunner,
        admin: &'a SigningAccount,
        emission_per_second: impl Into<Uint128>,
    ) -> DefaultRewardDistributorRobot<'a> {
        // Upload and instantiate astroport contracts
        let astro_robot = DefaultAstroportRobot::<TestRunner>::instantiate_local(
            runner,
            &admin,
            &Some("../../artifacts"),
            false,
            &None,
        );

        // Create reward pool
        let (reward_pool_addr, reward_lp_addr) = astro_robot.create_astroport_pair(
            PairType::Xyk {},
            &[
                AstroAssetInfo::NativeToken {
                    denom: "untrn".to_string(),
                },
                AstroAssetInfo::NativeToken {
                    denom: "uaxlr".to_string(),
                },
            ],
            None,
            &admin,
            Some(&[1000000000u128, 1000000000u128]),
            Some(&[6, 6]),
        );
        let reward_pool = cw_dex::Pool::Astroport(AstroportPool {
            lp_token_addr: Addr::unchecked(&reward_lp_addr),
            pair_addr: Addr::unchecked(&reward_pool_addr),
            pair_type: PairType::Xyk {},
            pool_assets: vec![AssetInfo::native("untrn"), AssetInfo::native("uaxlr")],
        });

        // Create Mock vault for reward pool
        let reward_vault_robot = DefaultVaultRobot::instantiate(
            runner,
            &admin,
            reward_lp_addr.as_str(),
            Some(Coin::from_str(DENOM_CREATION_FEE).unwrap()),
        );

        // Upload and instantiate reward distributor contract
        let code = Self::contract(runner);
        let code_id = runner.store_code(code, &admin).unwrap();
        let distribution_acc = runner.init_account(&[]).unwrap();
        let msg = InstantiateMsg {
            distribution_addr: distribution_acc.address(),
            emission_per_second: emission_per_second.into(),
            owner: admin.address(),
            reward_pool: reward_pool.clone(),
            reward_vault_addr: reward_vault_robot.vault_addr.clone(),
        };
        let contract_addr = Wasm::new(runner)
            .instantiate(code_id, &msg, Some(&admin.address()), None, &[], &admin)
            .unwrap()
            .data
            .address;

        Self::default_reward_distributor_robot(
            runner,
            astro_robot.astroport_contracts,
            contract_addr,
            distribution_acc,
            reward_pool_addr,
            reward_lp_addr,
            reward_vault_robot,
        )
    }

    fn query_config(&self) -> reward_distributor::msg::ConfigResponse {
        let query_msg = reward_distributor::msg::QueryMsg::Config {};
        self.wasm()
            .query(self.reward_distributor_addr(), &query_msg)
            .unwrap()
    }
}

/// A helper struct implementing the Robot testing pattern for testing the reward distributor
/// contract.
pub struct DefaultRewardDistributorRobot<'a> {
    pub runner: &'a TestRunner<'a>,
    pub astroport_contracts: AstroportContracts,
    pub reward_distributor_addr: String,
    pub distribution_acc: SigningAccount,
    pub reward_pool_addr: String,
    pub reward_lp_addr: String,
    pub reward_vault_robot: DefaultVaultRobot<'a, TestRunner<'a>>,
}

impl<'a> TestRobot<'a, TestRunner<'a>> for DefaultRewardDistributorRobot<'a> {
    fn runner(&self) -> &'a TestRunner<'a> {
        self.runner
    }
}

impl<'a> AstroportTestRobot<'a, TestRunner<'a>> for DefaultRewardDistributorRobot<'a> {
    fn astroport_contracts(&self) -> &AstroportContracts {
        &self.astroport_contracts
    }
}

impl<'a> RewardDistributorRobot<'a> for DefaultRewardDistributorRobot<'a> {
    fn reward_distributor_addr(&self) -> &str {
        &self.reward_distributor_addr
    }
}
