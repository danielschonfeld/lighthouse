use crate::BlockId;
use beacon_chain::{BeaconChain, BeaconChainTypes};
use std::str::FromStr;
use types::{BeaconState, Fork, Hash256, Slot};

#[derive(Debug)]
pub enum StateId {
    Head,
    Genesis,
    Finalized,
    Justified,
    Slot(Slot),
    Root(Hash256),
}

impl StateId {
    pub fn root<T: BeaconChainTypes>(
        &self,
        chain: &BeaconChain<T>,
    ) -> Result<Hash256, warp::Rejection> {
        let block = match self {
            StateId::Head => {
                return chain
                    .head_info()
                    .map(|head| head.state_root)
                    .map_err(crate::reject::beacon_chain_error)
            }
            StateId::Genesis => return Ok(chain.genesis_state_root),
            StateId::Finalized => BlockId::Finalized.block(chain),
            StateId::Justified => BlockId::Justified.block(chain),
            StateId::Slot(slot) => BlockId::Slot(*slot).block(chain),
            StateId::Root(root) => return Ok(*root),
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
        let (state_root, slot_opt) = match self {
            StateId::Head => {
                return chain
                    .head_beacon_state()
                    .map_err(crate::reject::beacon_chain_error)
            }
            StateId::Slot(slot) => (self.root(chain)?, Some(*slot)),
            other => (other.root(chain)?, None),
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
        let state = match self {
            StateId::Head => {
                return chain
                    .map_head(|snapshot| func(&snapshot.beacon_state))
                    .map_err(crate::reject::beacon_chain_error)?
            }
            other => other.state(chain)?,
        };

        func(&state)
    }
}

impl FromStr for StateId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "head" => Ok(StateId::Head),
            "genesis" => Ok(StateId::Genesis),
            "finalized" => Ok(StateId::Finalized),
            "justified" => Ok(StateId::Justified),
            other => {
                if other.starts_with("0x") {
                    Hash256::from_str(s)
                        .map(StateId::Root)
                        .map_err(|e| format!("{} cannot be parsed as a root", e))
                } else {
                    u64::from_str(s)
                        .map(Slot::new)
                        .map(StateId::Slot)
                        .map_err(|_| format!("{} cannot be parsed as a slot", s))
                }
            }
        }
    }
}
