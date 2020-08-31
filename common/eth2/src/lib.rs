pub mod types;

use self::types::*;
use reqwest::{Error, IntoUrl, StatusCode};
use serde::de::DeserializeOwned;

pub use reqwest::Url;

pub struct BeaconNodeClient {
    client: reqwest::Client,
    server: Url,
}

impl BeaconNodeClient {
    /// Returns `Err(())` if the URL is invalid.
    pub fn new(mut server: Url) -> Result<Self, ()> {
        server.path_segments_mut()?.push("eth").push("v1");

        Ok(Self {
            client: reqwest::Client::new(),
            server,
        })
    }

    async fn get<T: DeserializeOwned, U: IntoUrl>(&self, url: U) -> Result<T, Error> {
        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    async fn get_opt<T: DeserializeOwned, U: IntoUrl>(&self, url: U) -> Result<Option<T>, Error> {
        match self.client.get(url).send().await?.error_for_status() {
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

    /// `GET beacon/genesis`
    ///
    /// ## Errors
    ///
    /// May return a `404` if beacon chain genesis has not yet occurred.
    pub async fn beacon_genesis(&self) -> Result<GenericResponse<GenesisData>, Error> {
        let mut path = self.server.clone();

        path.path_segments_mut()
            .expect("path is base")
            .push("beacon")
            .push("genesis");

        self.get(path).await
    }

    /// `GET beacon/states/{state_id}/root`
    ///
    /// Returns `Ok(None)` on a 404 error.
    pub async fn beacon_states_root(
        &self,
        state_id: StateId,
    ) -> Result<Option<GenericResponse<RootData>>, Error> {
        let mut path = self.server.clone();

        path.path_segments_mut()
            .expect("path is base")
            .push("beacon")
            .push("states")
            .push(&state_id.to_string())
            .push("root");

        self.get_opt(path).await
    }

    /// `GET beacon/states/{state_id}/fork`
    ///
    /// Returns `Ok(None)` on a 404 error.
    pub async fn beacon_states_fork(
        &self,
        state_id: StateId,
    ) -> Result<Option<GenericResponse<Fork>>, Error> {
        let mut path = self.server.clone();

        path.path_segments_mut()
            .expect("path is base")
            .push("beacon")
            .push("states")
            .push(&state_id.to_string())
            .push("fork");

        self.get_opt(path).await
    }

    /// `GET beacon/states/{state_id}/finality_checkpoints`
    ///
    /// Returns `Ok(None)` on a 404 error.
    pub async fn beacon_states_finality_checkpoints(
        &self,
        state_id: StateId,
    ) -> Result<Option<GenericResponse<FinalityCheckpointsData>>, Error> {
        let mut path = self.server.clone();

        path.path_segments_mut()
            .expect("path is base")
            .push("beacon")
            .push("states")
            .push(&state_id.to_string())
            .push("finality_checkpoints");

        self.get_opt(path).await
    }

    /// `GET beacon/states/{state_id}/validators`
    ///
    /// Returns `Ok(None)` on a 404 error.
    pub async fn beacon_states_validators(
        &self,
        state_id: StateId,
    ) -> Result<Option<GenericResponse<Vec<ValidatorData>>>, Error> {
        let mut path = self.server.clone();

        path.path_segments_mut()
            .expect("path is base")
            .push("beacon")
            .push("states")
            .push(&state_id.to_string())
            .push("validators");

        self.get_opt(path).await
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
        let mut path = self.server.clone();

        path.path_segments_mut()
            .expect("path is base")
            .push("beacon")
            .push("states")
            .push(&state_id.to_string())
            .push("committees")
            .push(&epoch.to_string());

        if let Some(slot) = slot {
            path.query_pairs_mut()
                .append_pair("slot", &slot.to_string());
        }

        if let Some(index) = index {
            path.query_pairs_mut()
                .append_pair("index", &index.to_string());
        }

        self.get_opt(path).await
    }

    /// `GET beacon/states/{state_id}/validators/{validator_id}`
    ///
    /// Returns `Ok(None)` on a 404 error.
    pub async fn beacon_states_validator_id(
        &self,
        state_id: StateId,
        validator_id: &ValidatorId,
    ) -> Result<Option<GenericResponse<ValidatorData>>, Error> {
        let mut path = self.server.clone();

        path.path_segments_mut()
            .expect("path is base")
            .push("beacon")
            .push("states")
            .push(&state_id.to_string())
            .push("validators")
            .push(&validator_id.to_string());

        self.get_opt(path).await
    }

    /// `GET beacon/headers?slot,parent_root`
    ///
    /// Returns `Ok(None)` on a 404 error.
    pub async fn beacon_headers(
        &self,
        slot: Option<Slot>,
        parent_root: Option<u64>,
    ) -> Result<Option<GenericResponse<Vec<BlockHeaderData>>>, Error> {
        let mut path = self.server.clone();

        path.path_segments_mut()
            .expect("path is base")
            .push("beacon")
            .push("headers");

        if let Some(slot) = slot {
            path.query_pairs_mut()
                .append_pair("slot", &slot.to_string());
        }

        if let Some(root) = parent_root {
            path.query_pairs_mut()
                .append_pair("parent_root", &root.to_string());
        }

        self.get_opt(path).await
    }

    /// `GET beacon/blocks/{block_id}/root`
    ///
    /// Returns `Ok(None)` on a 404 error.
    pub async fn beacon_blocks_root(
        &self,
        block_id: BlockId,
    ) -> Result<Option<GenericResponse<RootData>>, Error> {
        let mut path = self.server.clone();

        path.path_segments_mut()
            .expect("path is base")
            .push("beacon")
            .push("blocks")
            .push(&block_id.to_string())
            .push("root");

        self.get_opt(path).await
    }
}
