mod block_id;
mod reject;
mod state_id;

use beacon_chain::{BeaconChain, BeaconChainTypes};
use block_id::BlockId;
use http_api_common as api;
use serde::Serialize;
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
        .and_then(|state_id: StateId, chain: Arc<BeaconChain<T>>| {
            blocking_json_task(move || {
                state_id
                    .root(&chain)
                    .map(api::RootData::from)
                    .map(api::GenericResponse::from)
            })
        });

    let beacon_state_fork = beacon_states_path
        .clone()
        .and(warp::path("fork"))
        .and(warp::path::end())
        .and_then(|state_id: StateId, chain: Arc<BeaconChain<T>>| {
            blocking_json_task(move || state_id.fork(&chain).map(api::GenericResponse::from))
        });

    let beacon_state_finality_checkpoints = beacon_states_path
        .clone()
        .and(warp::path("finality_checkpoints"))
        .and(warp::path::end())
        .and_then(|state_id: StateId, chain: Arc<BeaconChain<T>>| {
            blocking_json_task(move || {
                state_id
                    .map_state(&chain, |state| {
                        Ok(api::FinalityCheckpointsData {
                            previous_justified: state.previous_justified_checkpoint,
                            current_justified: state.current_justified_checkpoint,
                            finalized: state.finalized_checkpoint,
                        })
                    })
                    .map(api::GenericResponse::from)
            })
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
        .and_then(|block_id: BlockId, chain: Arc<BeaconChain<T>>| {
            blocking_json_task(move || {
                block_id
                    .root(&chain)
                    .map(api::RootData::from)
                    .map(api::GenericResponse::from)
            })
        });

    let routes = beacon_state_root
        .or(beacon_state_fork)
        .or(beacon_state_finality_checkpoints)
        .or(beacon_block_root);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn blocking_task<F, T>(func: F) -> T
where
    F: Fn() -> T,
{
    tokio::task::block_in_place(func)
}

async fn blocking_json_task<F, T>(func: F) -> Result<warp::reply::Json, warp::Rejection>
where
    F: Fn() -> Result<T, warp::Rejection>,
    T: Serialize,
{
    blocking_task(func)
        .await
        .map(|resp| warp::reply::json(&resp))
}
