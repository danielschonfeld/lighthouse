use beacon_chain::{BeaconChain, BeaconChainTypes};
use eth2::types::BlockId as CoreBlockId;
use std::str::FromStr;
use types::{Hash256, SignedBeaconBlock, Slot};

#[derive(Debug)]
pub struct BlockId(pub CoreBlockId);

impl BlockId {
    pub fn from_slot(slot: Slot) -> Self {
        Self(CoreBlockId::Slot(slot))
    }

    pub fn from_root(root: Hash256) -> Self {
        Self(CoreBlockId::Root(root))
    }

    pub fn root<T: BeaconChainTypes>(
        &self,
        chain: &BeaconChain<T>,
    ) -> Result<Hash256, warp::Rejection> {
        match &self.0 {
            CoreBlockId::Head => chain
                .head_info()
                .map(|head| head.block_root)
                .map_err(crate::reject::beacon_chain_error),
            CoreBlockId::Genesis => Ok(chain.genesis_block_root),
            CoreBlockId::Finalized => chain
                .head_info()
                .map(|head| head.finalized_checkpoint.root)
                .map_err(crate::reject::beacon_chain_error),
            CoreBlockId::Justified => chain
                .head_info()
                .map(|head| head.current_justified_checkpoint.root)
                .map_err(crate::reject::beacon_chain_error),
            CoreBlockId::Slot(slot) => chain
                .block_root_at_slot(*slot)
                .map_err(crate::reject::beacon_chain_error)
                .and_then(|root_opt| {
                    root_opt.ok_or_else(|| {
                        crate::reject::custom_not_found(format!("beacon block at slot {}", slot))
                    })
                }),
            CoreBlockId::Root(root) => Ok(*root),
        }
    }

    pub fn block<T: BeaconChainTypes>(
        &self,
        chain: &BeaconChain<T>,
    ) -> Result<SignedBeaconBlock<T::EthSpec>, warp::Rejection> {
        match &self.0 {
            CoreBlockId::Head => chain
                .head_beacon_block()
                .map_err(crate::reject::beacon_chain_error),
            _ => {
                let root = self.root(chain)?;
                chain
                    .get_block(&root)
                    .map_err(crate::reject::beacon_chain_error)
                    .and_then(|root_opt| {
                        root_opt.ok_or_else(|| {
                            crate::reject::custom_not_found(format!(
                                "beacon block with root {}",
                                root
                            ))
                        })
                    })
            }
        }
    }
}

impl FromStr for BlockId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        CoreBlockId::from_str(s).map(Self)
    }
}
