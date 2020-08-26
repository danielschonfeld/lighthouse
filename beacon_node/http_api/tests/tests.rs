use beacon_chain::{
    test_utils::{AttestationStrategy, BeaconChainHarness, BlockStrategy, HarnessType},
    BeaconChain,
};
use eth2::{types::StateId, BeaconNodeClient};
use http_api::Context;
use std::sync::Arc;
use store::config::StoreConfig;
use tokio::sync::oneshot;
use types::{test_utils::generate_deterministic_keypairs, MainnetEthSpec};

const VALIDATOR_COUNT: usize = 24;
const CHAIN_LENGTH: usize = 32 * 6;

pub struct ApiTester {
    chain: Arc<BeaconChain<HarnessType<MainnetEthSpec>>>,
    client: BeaconNodeClient,
    _server_shutdown: oneshot::Sender<()>,
}

impl ApiTester {
    pub fn new() -> Self {
        let harness = BeaconChainHarness::new(
            MainnetEthSpec,
            generate_deterministic_keypairs(VALIDATOR_COUNT),
            StoreConfig::default(),
        );

        harness.advance_slot();

        harness.extend_chain(
            CHAIN_LENGTH,
            BlockStrategy::OnCanonicalHead,
            AttestationStrategy::AllValidators,
        );

        let chain = Arc::new(harness.chain);

        let context = Arc::new(Context {
            chain: Some(chain.clone()),
            listen_address: [127, 0, 0, 1],
            listen_port: 0,
        });
        let ctx = context.clone();
        let (listening_socket, server, server_shutdown) = http_api::serve(ctx).unwrap();
        dbg!(listening_socket);
        tokio::spawn(async { server.await });

        let client = BeaconNodeClient::new(format!(
            "http://{}:{}",
            listening_socket.ip(),
            listening_socket.port()
        ));

        Self {
            chain,
            client,
            _server_shutdown: server_shutdown,
        }
    }

    pub async fn test_beacon_state_root(self, state_id: StateId) -> Self {
        let result = self.client.beacon_states_root(state_id).await.unwrap();

        let expected = match state_id {
            StateId::Head => self.chain.head_info().unwrap().state_root,
            _ => todo!(),
        };

        assert_eq!(result.data.root, expected);

        self
    }
}

#[tokio::test(core_threads = 2)]
async fn my_test() {
    ApiTester::new().test_beacon_state_root(StateId::Head).await;
}
