use beacon_chain::{BeaconChain, BeaconChainTypes};
use eth2::types::StateId as CoreStateId;
use std::str::FromStr;
use types::{BeaconState, EthSpec, Fork, Hash256};

pub struct StateId(CoreStateId);

impl StateId {
    pub fn root<T: BeaconChainTypes>(
        &self,
        chain: &BeaconChain<T>,
    ) -> Result<Hash256, warp::Rejection> {
        let slot = match &self.0 {
            CoreStateId::Head => {
                return chain
                    .head_info()
                    .map(|head| head.state_root)
                    .map_err(crate::reject::beacon_chain_error)
            }
            CoreStateId::Genesis => return Ok(chain.genesis_state_root),
            CoreStateId::Finalized => chain.head_info().map(|head| {
                head.finalized_checkpoint
                    .epoch
                    .start_slot(T::EthSpec::slots_per_epoch())
            }),
            CoreStateId::Justified => chain.head_info().map(|head| {
                head.current_justified_checkpoint
                    .epoch
                    .start_slot(T::EthSpec::slots_per_epoch())
            }),
            CoreStateId::Slot(slot) => Ok(*slot),
            CoreStateId::Root(root) => return Ok(*root),
        }
        .map_err(crate::reject::beacon_chain_error)?;

        chain
            .state_root_at_slot(slot)
            .map_err(crate::reject::beacon_chain_error)?
            .ok_or_else(|| {
                crate::reject::custom_not_found(format!("beacon state at slot {}", slot))
            })
    }

    pub fn fork<T: BeaconChainTypes>(
        &self,
        chain: &BeaconChain<T>,
    ) -> Result<Fork, warp::Rejection> {
        self.map_state(chain, |state| Ok(state.fork.clone()))
    }

    pub fn state<T: BeaconChainTypes>(
        &self,
        chain: &BeaconChain<T>,
    ) -> Result<BeaconState<T::EthSpec>, warp::Rejection> {
        let (state_root, slot_opt) = match &self.0 {
            CoreStateId::Head => {
                return chain
                    .head_beacon_state()
                    .map_err(crate::reject::beacon_chain_error)
            }
            CoreStateId::Slot(slot) => (self.root(chain)?, Some(*slot)),
            _ => (self.root(chain)?, None),
        };

        chain
            .get_state(&state_root, slot_opt)
            .map_err(crate::reject::beacon_chain_error)
            .and_then(|opt| {
                opt.ok_or_else(|| {
                    crate::reject::custom_not_found(format!("beacon state at root {}", state_root))
                })
            })
    }

    pub fn map_state<T: BeaconChainTypes, F, U>(
        &self,
        chain: &BeaconChain<T>,
        func: F,
    ) -> Result<U, warp::Rejection>
    where
        F: Fn(&BeaconState<T::EthSpec>) -> Result<U, warp::Rejection>,
    {
        match &self.0 {
            CoreStateId::Head => {
                return chain
                    .map_head(|snapshot| func(&snapshot.beacon_state))
                    .map_err(crate::reject::beacon_chain_error)?
            }
            _ => func(&self.state(chain)?),
        }
    }
}

impl FromStr for StateId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        CoreStateId::from_str(s).map(Self)
    }
}
