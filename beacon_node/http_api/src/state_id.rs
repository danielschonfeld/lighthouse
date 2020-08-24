use beacon_chain::{BeaconChain, BeaconChainTypes};
use std::str::FromStr;
use types::{Hash256, Slot};

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
        match self {
            StateId::Head => chain
                .head_info()
                .map(|head| head.state_root)
                .map_err(crate::reject::beacon_chain_error),
            StateId::Genesis => Ok(chain.genesis_state_root),
            StateId::Finalized => todo!(),
            StateId::Justified => todo!(),
            StateId::Slot(_) => todo!(),
            StateId::Root(root) => Ok(*root),
        }
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
