mod block_id;
mod reject;
mod state_id;

use beacon_chain::{BeaconChain, BeaconChainTypes};
use block_id::BlockId;
use state_id::StateId;
use std::sync::Arc;
use warp::Filter;

const API_PREFIX: &str = "eth";
const API_VERSION: &str = "v1";

pub struct Context<T: BeaconChainTypes> {
    pub chain: Option<Arc<BeaconChain<T>>>,
}

pub async fn serve<T: BeaconChainTypes>(ctx: Arc<Context<T>>) {
    let base_path = warp::path(API_PREFIX).and(warp::path(API_VERSION));
    let chain_filter = warp::any()
        .map(move || ctx.chain.clone())
        .and_then(|chain| async move {
            match chain {
                Some(chain) => Ok(chain),
                None => Err(warp::reject::not_found()),
            }
        });

    /*
     * beacon/states
     */

    let beacon_states_path = base_path
        .and(warp::path("beacon"))
        .and(warp::path("states"))
        .and(warp::path::param::<StateId>())
        .and(chain_filter.clone());

    let beacon_state_root = beacon_states_path
        .clone()
        .and(warp::path("root"))
        .and(warp::path::end())
        .and_then(|state_id: StateId, chain: Arc<BeaconChain<T>>| async move {
            state_id.root(&chain).map(|resp| warp::reply::json(&resp))
        });

    let beacon_state_fork = beacon_states_path
        .clone()
        .and(warp::path("fork"))
        .and(warp::path::end())
        .and_then(|state_id: StateId, chain: Arc<BeaconChain<T>>| async move {
            state_id.root(&chain).map(|resp| warp::reply::json(&resp))
        });

    /*
     * beacon/blocks
     */

    let beacon_blocks_path = base_path
        .and(warp::path("beacon"))
        .and(warp::path("blocks"))
        .and(warp::path::param::<BlockId>())
        .and(chain_filter.clone());

    let beacon_block_root = beacon_blocks_path
        .clone()
        .and(warp::path("root"))
        .and(warp::path::end())
        .and_then(|block_id: BlockId, chain: Arc<BeaconChain<T>>| async move {
            block_id.root(&chain).map(|resp| warp::reply::json(&resp))
        });

    let routes = beacon_state_root
        .or(beacon_state_fork)
        .or(beacon_block_root);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
