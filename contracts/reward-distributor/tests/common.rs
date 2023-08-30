use cw_it::cw_multi_test::{StargateKeeper, StargateMessageHandler};
use cw_it::multi_test::modules::TokenFactory;
use cw_it::multi_test::MultiTestRunner;
use cw_it::OwnedTestRunner;

pub use neutron_astroport_reward_distributor_test_helpers::robot::DENOM_CREATION_FEE;

pub const UNOPTIMIZED_PATH: &str = "../target/wasm32-unknown-unknown/release";
pub const DEPS_PATH: &str = "tests/artifacts";

/// The cw-multi-test mock of Token Factory.
pub const TOKEN_FACTORY: &TokenFactory =
    &TokenFactory::new("factory", 32, 16, 59 + 16, DENOM_CREATION_FEE);

/// Helper function to get a test runner based on env var `TEST_RUNNER_TYPE`.
pub fn get_test_runner<'a>() -> OwnedTestRunner<'a> {
    match option_env!("TEST_RUNNER_TYPE").unwrap_or("multi-test") {
        "multi-test" => {
            let mut stargate_keeper = StargateKeeper::new();
            TOKEN_FACTORY.register_msgs(&mut stargate_keeper);

            OwnedTestRunner::MultiTest(MultiTestRunner::new_with_stargate("osmo", stargate_keeper))
        }
        #[cfg(feature = "osmosis-test-tube")]
        "osmosis-test-app" => {
            OwnedTestRunner::OsmosisTestApp(cw_it::osmosis_test_tube::OsmosisTestApp::new())
        }
        _ => panic!("Unsupported test runner type"),
    }
}
