use beacon_chain::{
    test_utils::{AttestationStrategy, BeaconChainHarness, BlockStrategy, HarnessType},
    BeaconChain,
};
use eth2::{types::StateId, BeaconNodeClient};
use http_api::Context;
use std::sync::Arc;
use store::config::StoreConfig;
use tokio::sync::oneshot;
use types::{test_utils::generate_deterministic_keypairs, EthSpec, MainnetEthSpec, Slot};

type E = MainnetEthSpec;

const VALIDATOR_COUNT: usize = 24;
const SLOTS_PER_EPOCH: u64 = 32;
const CHAIN_LENGTH: u64 = SLOTS_PER_EPOCH * 5;
const JUSTIFIED_EPOCH: u64 = 4;
const FINALIZED_EPOCH: u64 = 3;

/// Skipping the slots around the epoch boundary allows us to check that we're obtaining states
/// from skipped slots for the finalized and justified checkpoints (instead of the state from the
/// block that those roots point to).
const SKIPPED_SLOTS: &[u64] = &[
    JUSTIFIED_EPOCH * SLOTS_PER_EPOCH - 1,
    JUSTIFIED_EPOCH * SLOTS_PER_EPOCH,
    FINALIZED_EPOCH * SLOTS_PER_EPOCH - 1,
    FINALIZED_EPOCH * SLOTS_PER_EPOCH,
];

pub struct ApiTester {
    chain: Arc<BeaconChain<HarnessType<E>>>,
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

        for _ in 0..CHAIN_LENGTH {
            let slot = harness.chain.slot().unwrap().as_u64();

            if !SKIPPED_SLOTS.contains(&slot) {
                harness.extend_chain(
                    1,
                    BlockStrategy::OnCanonicalHead,
                    AttestationStrategy::AllValidators,
                );
            }

            harness.advance_slot();
        }

        let chain = Arc::new(harness.chain);

        assert_eq!(
            chain.head_info().unwrap().finalized_checkpoint.epoch,
            3,
            "precondition: finality"
        );
        assert_eq!(
            chain
                .head_info()
                .unwrap()
                .current_justified_checkpoint
                .epoch,
            4,
            "precondition: justification"
        );

        let context = Arc::new(Context {
            chain: Some(chain.clone()),
            listen_address: [127, 0, 0, 1],
            listen_port: 0,
        });
        let ctx = context.clone();
        let (listening_socket, server, server_shutdown) = http_api::serve(ctx).unwrap();

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
            StateId::Genesis => self.chain.genesis_state_root,
            StateId::Finalized => {
                let finalized_slot = self
                    .chain
                    .head_info()
                    .unwrap()
                    .finalized_checkpoint
                    .epoch
                    .start_slot(E::slots_per_epoch());

                self.chain
                    .state_root_at_slot(finalized_slot)
                    .unwrap()
                    .unwrap()
            }
            StateId::Justified => {
                let justified_slot = self
                    .chain
                    .head_info()
                    .unwrap()
                    .current_justified_checkpoint
                    .epoch
                    .start_slot(E::slots_per_epoch());

                self.chain
                    .state_root_at_slot(justified_slot)
                    .unwrap()
                    .unwrap()
            }
            StateId::Slot(slot) => self.chain.state_root_at_slot(slot).unwrap().unwrap(),
            StateId::Root(root) => root,
        };

        assert_eq!(result.data.root, expected, "{:?}", state_id);

        self
    }
}

#[tokio::test(core_threads = 2)]
async fn beacon_state_root() {
    ApiTester::new()
        .test_beacon_state_root(StateId::Head)
        .await
        .test_beacon_state_root(StateId::Genesis)
        .await
        .test_beacon_state_root(StateId::Finalized)
        .await
        .test_beacon_state_root(StateId::Justified)
        .await
        .test_beacon_state_root(StateId::Slot(Slot::new(0)))
        .await
        .test_beacon_state_root(StateId::Slot(Slot::new(32)))
        .await
        .test_beacon_state_root(StateId::Slot(Slot::from(SKIPPED_SLOTS[0])))
        .await
        .test_beacon_state_root(StateId::Slot(Slot::from(SKIPPED_SLOTS[1])))
        .await
        .test_beacon_state_root(StateId::Slot(Slot::from(SKIPPED_SLOTS[2])))
        .await
        .test_beacon_state_root(StateId::Slot(Slot::from(SKIPPED_SLOTS[3])))
        .await;
}
