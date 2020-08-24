use warp::reject::Reject;

pub fn beacon_chain_error(e: beacon_chain::BeaconChainError) -> warp::reject::Rejection {
    warp::reject::custom(BeaconChainError(e))
}

#[derive(Debug)]
pub struct BeaconChainError(pub beacon_chain::BeaconChainError);

impl Reject for BeaconChainError {}
