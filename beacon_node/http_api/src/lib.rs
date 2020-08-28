mod block_id;
mod reject;
mod state_id;

use beacon_chain::{BeaconChain, BeaconChainError, BeaconChainTypes};
use block_id::BlockId;
use eth2::types::{self as api_types, ValidatorId};
use serde::{Deserialize, Serialize};
use state_id::StateId;
use std::borrow::Cow;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::oneshot;
use types::{CommitteeCache, Epoch, EthSpec, RelativeEpoch, Slot};
use warp::Filter;

const API_PREFIX: &str = "eth";
const API_VERSION: &str = "v1";

pub struct Context<T: BeaconChainTypes> {
    pub chain: Option<Arc<BeaconChain<T>>>,
    pub listen_address: [u8; 4],
    pub listen_port: u16,
}

pub fn serve<T: BeaconChainTypes>(
    ctx: Arc<Context<T>>,
) -> Result<(SocketAddr, impl Future<Output = ()>, oneshot::Sender<()>), warp::Error> {
    let listen_address = ctx.listen_address;
    let listen_port = ctx.listen_port;

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

    // beacon/states/{state_id}/root
    let beacon_state_root = beacon_states_path
        .clone()
        .and(warp::path("root"))
        .and(warp::path::end())
        .and_then(|state_id: StateId, chain: Arc<BeaconChain<T>>| {
            blocking_json_task(move || {
                state_id
                    .root(&chain)
                    .map(api_types::RootData::from)
                    .map(api_types::GenericResponse::from)
            })
        });

    // beacon/states/{state_id}/fork
    let beacon_state_fork = beacon_states_path
        .clone()
        .and(warp::path("fork"))
        .and(warp::path::end())
        .and_then(|state_id: StateId, chain: Arc<BeaconChain<T>>| {
            blocking_json_task(move || state_id.fork(&chain).map(api_types::GenericResponse::from))
        });

    // beacon/states/{state_id}/finality_checkpoints
    let beacon_state_finality_checkpoints = beacon_states_path
        .clone()
        .and(warp::path("finality_checkpoints"))
        .and(warp::path::end())
        .and_then(|state_id: StateId, chain: Arc<BeaconChain<T>>| {
            blocking_json_task(move || {
                state_id
                    .map_state(&chain, |state| {
                        Ok(api_types::FinalityCheckpointsData {
                            previous_justified: state.previous_justified_checkpoint,
                            current_justified: state.current_justified_checkpoint,
                            finalized: state.finalized_checkpoint,
                        })
                    })
                    .map(api_types::GenericResponse::from)
            })
        });

    // beacon/states/{state_id}/validators
    let beacon_state_validators = beacon_states_path
        .clone()
        .and(warp::path("validators"))
        .and(warp::path::end())
        .and_then(|state_id: StateId, chain: Arc<BeaconChain<T>>| {
            blocking_json_task(move || {
                state_id
                    .map_state(&chain, |state| {
                        let epoch = state.current_epoch();
                        let finalized_epoch = state.finalized_checkpoint.epoch;
                        let far_future_epoch = chain.spec.far_future_epoch;

                        Ok(state
                            .validators
                            .iter()
                            .zip(state.balances.iter())
                            .enumerate()
                            .map(|(index, (validator, balance))| api_types::ValidatorData {
                                index: index as u64,
                                balance: *balance,
                                status: api_types::ValidatorStatus::from_validator(
                                    Some(validator),
                                    epoch,
                                    finalized_epoch,
                                    far_future_epoch,
                                ),
                                validator: validator.clone(),
                            })
                            .collect::<Vec<_>>())
                    })
                    .map(api_types::GenericResponse::from)
            })
        });

    // beacon/states/{state_id}/validators/{validator_id}
    let beacon_state_validators_id = beacon_states_path
        .clone()
        .and(warp::path("validators"))
        .and(warp::path::param::<ValidatorId>())
        .and(warp::path::end())
        .and_then(
            |state_id: StateId, chain: Arc<BeaconChain<T>>, validator_id: ValidatorId| {
                blocking_json_task(move || {
                    state_id
                        .map_state(&chain, |state| {
                            let index_opt = match &validator_id {
                                ValidatorId::PublicKey(pubkey) => {
                                    state.validators.iter().position(|v| v.pubkey == *pubkey)
                                }
                                ValidatorId::Index(index) => Some(*index as usize),
                            };

                            index_opt
                                .and_then(|index| {
                                    let validator = state.validators.get(index)?;
                                    let balance = *state.balances.get(index)?;
                                    let epoch = state.current_epoch();
                                    let finalized_epoch = state.finalized_checkpoint.epoch;
                                    let far_future_epoch = chain.spec.far_future_epoch;

                                    Some(api_types::ValidatorData {
                                        index: index as u64,
                                        balance,
                                        status: api_types::ValidatorStatus::from_validator(
                                            Some(validator),
                                            epoch,
                                            finalized_epoch,
                                            far_future_epoch,
                                        ),
                                        validator: validator.clone(),
                                    })
                                })
                                .ok_or_else(|| warp::reject::not_found())
                        })
                        .map(api_types::GenericResponse::from)
                })
            },
        );

    #[derive(Serialize, Deserialize)]
    struct CommitteesQuery {
        slot: Option<Slot>,
        index: Option<u64>,
    }

    // beacon/states/{state_id}/committees/{epoch}
    let beacon_state_committees = beacon_states_path
        .clone()
        .and(warp::path("committees"))
        .and(warp::path::param::<Epoch>())
        .and(warp::query::<CommitteesQuery>())
        .and(warp::path::end())
        .and_then(
            |state_id: StateId,
             chain: Arc<BeaconChain<T>>,
             epoch: Epoch,
             query: CommitteesQuery| {
                blocking_json_task(move || {
                    state_id.map_state(&chain, |state| {
                        let relative_epoch =
                            RelativeEpoch::from_epoch(state.current_epoch(), epoch).map_err(
                                |_| {
                                    crate::reject::custom_bad_request(format!(
                                        "only previous, current and next epochs are supported"
                                    ))
                                },
                            )?;

                        let committee_cache = if state
                            .committee_cache_is_initialized(relative_epoch)
                        {
                            state.committee_cache(relative_epoch).map(Cow::Borrowed)
                        } else {
                            CommitteeCache::initialized(state, epoch, &chain.spec).map(Cow::Owned)
                        }
                        .map_err(BeaconChainError::BeaconStateError)
                        .map_err(crate::reject::beacon_chain_error)?;

                        // Use either the supplied slot or all slots in the epoch.
                        let slots = query.slot.map(|slot| vec![slot]).unwrap_or_else(|| {
                            epoch.slot_iter(T::EthSpec::slots_per_epoch()).collect()
                        });

                        // Use either the supplied committee index or all available indices.
                        let indices = query.index.map(|index| vec![index]).unwrap_or_else(|| {
                            (0..committee_cache.committees_per_slot()).collect()
                        });

                        let mut response = Vec::with_capacity(slots.len() * indices.len());

                        for slot in slots {
                            // It is not acceptable to query with a slot that is not within the
                            // specified epoch.
                            if slot.epoch(T::EthSpec::slots_per_epoch()) != epoch {
                                return Err(crate::reject::custom_bad_request(format!(
                                    "{} is not in epoch {}",
                                    slot, epoch
                                )));
                            }

                            for &index in &indices {
                                let committee = committee_cache
                                    .get_beacon_committee(slot, index)
                                    .ok_or_else(|| {
                                    crate::reject::custom_bad_request(format!(
                                        "committee index {} does not exist in epoch {}",
                                        index, epoch
                                    ))
                                })?;

                                response.push(api_types::CommitteeData {
                                    index,
                                    slot,
                                    validators: committee
                                        .committee
                                        .into_iter()
                                        .map(|i| *i as u64)
                                        .collect(),
                                });
                            }
                        }

                        Ok(api_types::GenericResponse::from(response))
                    })
                })
            },
        );

    /*
     * beacon/blocks
     */

    let beacon_blocks_path = base_path
        .and(warp::path("beacon"))
        .and(warp::path("blocks"))
        .and(warp::path::param::<BlockId>())
        .and(chain_filter.clone());

    // beacon/blocks/{block_id}/root
    let beacon_block_root = beacon_blocks_path
        .clone()
        .and(warp::path("root"))
        .and(warp::path::end())
        .and_then(|block_id: BlockId, chain: Arc<BeaconChain<T>>| {
            blocking_json_task(move || {
                block_id
                    .root(&chain)
                    .map(api_types::RootData::from)
                    .map(api_types::GenericResponse::from)
            })
        });

    let routes = beacon_state_root
        .or(beacon_state_fork)
        .or(beacon_state_finality_checkpoints)
        .or(beacon_state_validators)
        .or(beacon_state_validators_id)
        .or(beacon_block_root)
        .or(beacon_state_committees)
        .recover(crate::reject::handle_rejection);

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let (listening_socket, server) = warp::serve(routes).try_bind_with_graceful_shutdown(
        (listen_address, listen_port),
        async {
            shutdown_rx.await.ok();
        },
    )?;

    Ok((listening_socket, server, shutdown_tx))
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
