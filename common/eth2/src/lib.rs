pub mod types;

use self::types::*;
use reqwest::{Error, StatusCode};
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
}
