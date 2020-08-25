use crate::BlockId;
use beacon_chain::{BeaconChain, BeaconChainTypes};
use eth2::types::{BlockId as CoreBlockId, StateId as CoreStateId};
use std::str::FromStr;
use types::{BeaconState, Fork, Hash256};

pub struct StateId(CoreStateId);

impl StateId {
    pub fn root<T: BeaconChainTypes>(
        &self,
        chain: &BeaconChain<T>,
    ) -> Result<Hash256, warp::Rejection> {
        let block = match &self.0 {
            CoreStateId::Head => {
                return chain
                    .head_info()
                    .map(|head| head.state_root)
                    .map_err(crate::reject::beacon_chain_error)
            }
            CoreStateId::Genesis => return Ok(chain.genesis_state_root),
            CoreStateId::Finalized => BlockId(CoreBlockId::Finalized).block(chain),
            CoreStateId::Justified => BlockId(CoreBlockId::Justified).block(chain),
            CoreStateId::Slot(slot) => BlockId(CoreBlockId::Slot(*slot)).block(chain),
            CoreStateId::Root(root) => return Ok(*root),
        }?;

        Ok(block.state_root())
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
            .and_then(|opt| opt.ok_or_else(|| warp::reject::not_found()))
    }

    pub fn map_state<T: BeaconChainTypes, F, U>(
        &self,
        chain: &BeaconChain<T>,
        func: F,
    ) -> Result<U, warp::Rejection>
    where
        F: Fn(&BeaconState<T::EthSpec>) -> Result<U, warp::Rejection>,
    {
        let state = match &self.0 {
            CoreStateId::Head => {
                return chain
                    .map_head(|snapshot| func(&snapshot.beacon_state))
                    .map_err(crate::reject::beacon_chain_error)?
            }
            _ => self.state(chain)?,
        };

        func(&state)
    }
}

impl FromStr for StateId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        CoreStateId::from_str(s).map(Self)
    }
}
