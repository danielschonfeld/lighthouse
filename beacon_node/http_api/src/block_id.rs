use beacon_chain::{BeaconChain, BeaconChainTypes};
use std::str::FromStr;
use types::{Hash256, Slot};

#[derive(Debug)]
pub enum BlockId {
    Head,
    Genesis,
    Finalized,
    Justified,
    Slot(Slot),
    Root(Hash256),
}

impl BlockId {
    pub fn root<T: BeaconChainTypes>(
        &self,
        chain: &BeaconChain<T>,
    ) -> Result<Hash256, warp::Rejection> {
        match self {
            BlockId::Head => chain
                .head_info()
                .map(|head| head.block_root)
                .map_err(crate::reject::beacon_chain_error),
            BlockId::Genesis => Ok(chain.genesis_block_root),
            BlockId::Finalized => chain
                .head_info()
                .map(|head| head.finalized_checkpoint.root)
                .map_err(crate::reject::beacon_chain_error),
            BlockId::Justified => chain
                .head_info()
                .map(|head| head.current_justified_checkpoint.root)
                .map_err(crate::reject::beacon_chain_error),
            BlockId::Slot(slot) => chain
                .block_root_at_slot(*slot)
                .map_err(crate::reject::beacon_chain_error)
                .and_then(|root_opt| root_opt.ok_or_else(|| warp::reject::not_found())),
            BlockId::Root(root) => Ok(*root),
        }
    }
}

impl FromStr for BlockId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "head" => Ok(BlockId::Head),
            "genesis" => Ok(BlockId::Genesis),
            "finalized" => Ok(BlockId::Finalized),
            "justified" => Ok(BlockId::Justified),
            other => {
                if other.starts_with("0x") {
                    Hash256::from_str(s)
                        .map(BlockId::Root)
                        .map_err(|e| format!("{} cannot be parsed as a root", e))
                } else {
                    u64::from_str(s)
                        .map(Slot::new)
                        .map(BlockId::Slot)
                        .map_err(|_| format!("{} cannot be parsed as a parameter", s))
                }
            }
        }
    }
}
