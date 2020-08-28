pub mod types;

use self::types::*;
use reqwest::{Error, StatusCode, Url};
use serde::de::DeserializeOwned;

const VERSION: &str = "eth/v1";

pub struct BeaconNodeClient {
    client: reqwest::Client,
    server: String,
}

impl BeaconNodeClient {
    pub fn new(server: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            server,
        }
    }

    pub fn from_parts(client: reqwest::Client, server: String) -> Self {
        Self { client, server }
    }

    fn path(&self, path: &str) -> String {
        format!("{}/{}/{}", self.server, VERSION, path)
    }

    async fn get_opt<T: DeserializeOwned>(&self, path: &str) -> Result<Option<T>, Error> {
        match self
            .client
            .get(&self.path(path))
            .send()
            .await?
            .error_for_status()
        {
            Ok(resp) => resp.json().await.map(Option::Some),
            Err(err) => {
                if err.status() == Some(StatusCode::NOT_FOUND) {
                    Ok(None)
                } else {
                    Err(err)
                }
            }
        }
    }

    /// `GET beacon/states/{state_id}/root`
    ///
    /// Returns `Ok(None)` on a 404 error.
    pub async fn beacon_states_root(
        &self,
        state_id: StateId,
    ) -> Result<Option<GenericResponse<RootData>>, Error> {
        self.get_opt(&format!("beacon/states/{}/root", state_id))
            .await
    }

    /// `GET beacon/states/{state_id}/fork`
    ///
    /// Returns `Ok(None)` on a 404 error.
    pub async fn beacon_states_fork(
        &self,
        state_id: StateId,
    ) -> Result<Option<GenericResponse<Fork>>, Error> {
        self.get_opt(&format!("beacon/states/{}/fork", state_id))
            .await
    }

    /// `GET beacon/states/{state_id}/finality_checkpoints`
    ///
    /// Returns `Ok(None)` on a 404 error.
    pub async fn beacon_states_finality_checkpoints(
        &self,
        state_id: StateId,
    ) -> Result<Option<GenericResponse<FinalityCheckpointsData>>, Error> {
        self.get_opt(&format!("beacon/states/{}/finality_checkpoints", state_id))
            .await
    }

    /// `GET beacon/states/{state_id}/validators`
    ///
    /// Returns `Ok(None)` on a 404 error.
    pub async fn beacon_states_validators(
        &self,
        state_id: StateId,
    ) -> Result<Option<GenericResponse<Vec<ValidatorData>>>, Error> {
        self.get_opt(&format!("beacon/states/{}/validators", state_id))
            .await
    }

    /// `GET beacon/states/{state_id}/committees?slot,index`
    ///
    /// Returns `Ok(None)` on a 404 error.
    pub async fn beacon_states_committees(
        &self,
        state_id: StateId,
        epoch: Epoch,
        slot: Option<Slot>,
        index: Option<u64>,
    ) -> Result<Option<GenericResponse<Vec<CommitteeData>>>, Error> {
        let mut path = Url::parse(&format!("beacon/states/{}/committees/{}", state_id, epoch))
            .expect("url should always be valid");

        if let Some(slot) = slot {
            path.query_pairs_mut()
                .append_pair("slot", &slot.to_string());
        }

        if let Some(index) = index {
            path.query_pairs_mut()
                .append_pair("index", &index.to_string());
        }

        self.get_opt(&path.to_string()).await
    }

    /// `GET beacon/states/{state_id}/validators/{validator_id}`
    ///
    /// Returns `Ok(None)` on a 404 error.
    pub async fn beacon_states_validator_id(
        &self,
        state_id: StateId,
        validator_id: &ValidatorId,
    ) -> Result<Option<GenericResponse<ValidatorData>>, Error> {
        self.get_opt(&format!(
            "beacon/states/{}/validators/{}",
            state_id, validator_id
        ))
        .await
    }

    /// `GET beacon/blocks/{block_id}/root`
    ///
    /// Returns `Ok(None)` on a 404 error.
    pub async fn beacon_blocks_root(
        &self,
        block_id: BlockId,
    ) -> Result<Option<GenericResponse<RootData>>, Error> {
        self.get_opt(&format!("beacon/blocks/{}/root", block_id))
            .await
    }
}
