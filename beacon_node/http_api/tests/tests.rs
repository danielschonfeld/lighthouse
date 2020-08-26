use beacon_chain::{
    test_utils::{AttestationStrategy, BeaconChainHarness, BlockStrategy, HarnessType},
    BeaconChain,
};
use eth2::{types::*, BeaconNodeClient};
use http_api::Context;
use std::sync::Arc;
use store::config::StoreConfig;
use tokio::sync::oneshot;
use types::{
    test_utils::generate_deterministic_keypairs, BeaconState, EthSpec, Hash256, MainnetEthSpec,
    Slot,
};

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

    fn get_state(&self, state_id: StateId) -> Option<BeaconState<E>> {
        match state_id {
            StateId::Head => Some(self.chain.head().unwrap().beacon_state),
            StateId::Genesis => self
                .chain
                .get_state(&self.chain.genesis_state_root, None)
                .unwrap(),
            StateId::Finalized => {
                let finalized_slot = self
                    .chain
                    .head_info()
                    .unwrap()
                    .finalized_checkpoint
                    .epoch
                    .start_slot(E::slots_per_epoch());

                let root = self
                    .chain
                    .state_root_at_slot(finalized_slot)
                    .unwrap()
                    .unwrap();

                self.chain.get_state(&root, Some(finalized_slot)).unwrap()
            }
            StateId::Justified => {
                let justified_slot = self
                    .chain
                    .head_info()
                    .unwrap()
                    .current_justified_checkpoint
                    .epoch
                    .start_slot(E::slots_per_epoch());

                let root = self
                    .chain
                    .state_root_at_slot(justified_slot)
                    .unwrap()
                    .unwrap();

                self.chain.get_state(&root, Some(justified_slot)).unwrap()
            }
            StateId::Slot(slot) => {
                let root = self.chain.state_root_at_slot(slot).unwrap().unwrap();

                self.chain.get_state(&root, Some(slot)).unwrap()
            }
            StateId::Root(root) => self.chain.get_state(&root, None).unwrap(),
        }
    }

    pub async fn test_beacon_states_root(self, state_ids: &[StateId]) -> Self {
        for &state_id in state_ids {
            let result = self
                .client
                .beacon_states_root(state_id)
                .await
                .unwrap()
                .map(|res| res.data.root);

            let expected = match state_id {
                StateId::Head => Some(self.chain.head_info().unwrap().state_root),
                StateId::Genesis => Some(self.chain.genesis_state_root),
                StateId::Finalized => {
                    let finalized_slot = self
                        .chain
                        .head_info()
                        .unwrap()
                        .finalized_checkpoint
                        .epoch
                        .start_slot(E::slots_per_epoch());

                    self.chain.state_root_at_slot(finalized_slot).unwrap()
                }
                StateId::Justified => {
                    let justified_slot = self
                        .chain
                        .head_info()
                        .unwrap()
                        .current_justified_checkpoint
                        .epoch
                        .start_slot(E::slots_per_epoch());

                    self.chain.state_root_at_slot(justified_slot).unwrap()
                }
                StateId::Slot(slot) => self.chain.state_root_at_slot(slot).unwrap(),
                StateId::Root(root) => Some(root),
            };

            assert_eq!(result, expected, "{:?}", state_id);
        }

        self
    }

    pub async fn test_beacon_states_fork(self, state_ids: &[StateId]) -> Self {
        for &state_id in state_ids {
            let result = self
                .client
                .beacon_states_fork(state_id)
                .await
                .unwrap()
                .map(|res| res.data);

            let expected = self.get_state(state_id).map(|state| state.fork);

            assert_eq!(result, expected, "{:?}", state_id);
        }

        self
    }

    pub async fn test_beacon_states_finality_checkpoints(self, state_ids: &[StateId]) -> Self {
        for &state_id in state_ids {
            let result = self
                .client
                .beacon_states_finality_checkpoints(state_id)
                .await
                .unwrap()
                .map(|res| res.data);

            let expected = self
                .get_state(state_id)
                .map(|state| FinalityCheckpointsData {
                    previous_justified: state.previous_justified_checkpoint,
                    current_justified: state.current_justified_checkpoint,
                    finalized: state.finalized_checkpoint,
                });

            assert_eq!(result, expected, "{:?}", state_id);
        }

        self
    }

    pub async fn test_beacon_states_validators(self, state_ids: &[StateId]) -> Self {
        for &state_id in state_ids {
            let result = self
                .client
                .beacon_states_validators(state_id)
                .await
                .unwrap()
                .map(|res| res.data);

            let expected = self.get_state(state_id).map(|state| {
                let epoch = state.current_epoch();
                let finalized_epoch = state.finalized_checkpoint.epoch;
                let far_future_epoch = self.chain.spec.far_future_epoch;

                let mut validators = Vec::with_capacity(state.validators.len());

                for i in 0..state.validators.len() {
                    let validator = state.validators[i].clone();

                    validators.push(ValidatorData {
                        index: i as u64,
                        balance: state.balances[i],
                        status: ValidatorStatus::from_validator(
                            Some(&validator),
                            epoch,
                            finalized_epoch,
                            far_future_epoch,
                        ),
                        validator,
                    })
                }

                validators
            });

            assert_eq!(result, expected, "{:?}", state_id);
        }

        self
    }

    fn get_block_root(&self, block_id: BlockId) -> Option<Hash256> {
        match block_id {
            BlockId::Head => Some(self.chain.head_info().unwrap().block_root),
            BlockId::Genesis => Some(self.chain.genesis_block_root),
            BlockId::Finalized => Some(self.chain.head_info().unwrap().finalized_checkpoint.root),
            BlockId::Justified => Some(
                self.chain
                    .head_info()
                    .unwrap()
                    .current_justified_checkpoint
                    .root,
            ),
            BlockId::Slot(slot) => self.chain.block_root_at_slot(slot).unwrap(),
            BlockId::Root(root) => Some(root),
        }
    }

    pub async fn test_beacon_blocks_root(self, block_ids: &[BlockId]) -> Self {
        for &block_id in block_ids {
            let result = self
                .client
                .beacon_blocks_root(block_id)
                .await
                .unwrap()
                .map(|res| res.data.root);

            let expected = self.get_block_root(block_id);

            assert_eq!(result, expected, "{:?}", block_id);
        }

        self
    }
}

fn interesting_state_ids() -> Vec<StateId> {
    vec![
        StateId::Head,
        StateId::Genesis,
        StateId::Finalized,
        StateId::Justified,
        StateId::Slot(Slot::new(0)),
        StateId::Slot(Slot::new(32)),
        StateId::Slot(Slot::from(SKIPPED_SLOTS[0])),
        StateId::Slot(Slot::from(SKIPPED_SLOTS[1])),
        StateId::Slot(Slot::from(SKIPPED_SLOTS[2])),
        StateId::Slot(Slot::from(SKIPPED_SLOTS[3])),
        StateId::Root(Hash256::zero()),
    ]
}

fn interesting_block_ids() -> Vec<BlockId> {
    vec![
        BlockId::Head,
        BlockId::Genesis,
        BlockId::Finalized,
        BlockId::Justified,
        BlockId::Slot(Slot::new(0)),
        BlockId::Slot(Slot::new(32)),
        BlockId::Slot(Slot::from(SKIPPED_SLOTS[0])),
        BlockId::Slot(Slot::from(SKIPPED_SLOTS[1])),
        BlockId::Slot(Slot::from(SKIPPED_SLOTS[2])),
        BlockId::Slot(Slot::from(SKIPPED_SLOTS[3])),
        BlockId::Root(Hash256::zero()),
    ]
}

#[tokio::test(core_threads = 2)]
async fn beacon_states_root() {
    ApiTester::new()
        .test_beacon_states_root(&interesting_state_ids())
        .await;
}

#[tokio::test(core_threads = 2)]
async fn beacon_states_fork() {
    ApiTester::new()
        .test_beacon_states_fork(&interesting_state_ids())
        .await;
}

#[tokio::test(core_threads = 2)]
async fn beacon_states_finality_checkpoints() {
    ApiTester::new()
        .test_beacon_states_finality_checkpoints(&interesting_state_ids())
        .await;
}

#[tokio::test(core_threads = 2)]
async fn beacon_states_finality_validators() {
    ApiTester::new()
        .test_beacon_states_validators(&interesting_state_ids())
        .await;
}

#[tokio::test(core_threads = 2)]
async fn beacon_blocks_root() {
    ApiTester::new()
        .test_beacon_blocks_root(&interesting_block_ids())
        .await;
}
