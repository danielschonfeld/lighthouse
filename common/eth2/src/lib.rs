pub mod types;

use self::types::*;
use reqwest::{IntoUrl, Response, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use std::convert::TryFrom;

pub use reqwest::Url;

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    ServerMessage(ErrorMessage),
    StatusCode(StatusCode),
}

impl Error {
    fn status(&self) -> Option<StatusCode> {
        match self {
            Error::Reqwest(error) => error.status(),
            Error::ServerMessage(msg) => StatusCode::try_from(msg.code).ok(),
            Error::StatusCode(status) => Some(*status),
        }
    }
}

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
        let response = self.client.get(url).send().await.map_err(Error::Reqwest)?;
        ok_or_error(response)
            .await?
            .json()
            .await
            .map_err(Error::Reqwest)
    }

    async fn get_opt<T: DeserializeOwned, U: IntoUrl>(&self, url: U) -> Result<Option<T>, Error> {
        let response = self.client.get(url).send().await.map_err(Error::Reqwest)?;
        match ok_or_error(response).await {
            Ok(resp) => resp.json().await.map(Option::Some).map_err(Error::Reqwest),
            Err(err) => {
                if err.status() == Some(StatusCode::NOT_FOUND) {
                    Ok(None)
                } else {
                    Err(err)
                }
            }
        }
    }

    async fn post<T: Serialize, U: IntoUrl>(&self, url: U, body: &T) -> Result<(), Error> {
        let response = self
            .client
            .post(url)
            .json(body)
            .send()
            .await
            .map_err(Error::Reqwest)?;
        ok_or_error(response).await?;
        Ok(())
    }

    /// `GET beacon/genesis`
    ///
    /// ## Errors
    ///
    /// May return a `404` if beacon chain genesis has not yet occurred.
    pub async fn get_beacon_genesis(&self) -> Result<GenericResponse<GenesisData>, Error> {
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
    pub async fn get_beacon_states_root(
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
    pub async fn get_beacon_states_fork(
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
    pub async fn get_beacon_states_finality_checkpoints(
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
    pub async fn get_beacon_states_validators(
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
    pub async fn get_beacon_states_committees(
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
    pub async fn get_beacon_states_validator_id(
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
    pub async fn get_beacon_headers(
        &self,
        slot: Option<Slot>,
        parent_root: Option<Hash256>,
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
                .append_pair("parent_root", &format!("{:?}", root));
        }

        self.get_opt(path).await
    }

    /// `GET beacon/headers/{block_id}`
    ///
    /// Returns `Ok(None)` on a 404 error.
    pub async fn get_beacon_headers_block_id(
        &self,
        block_id: BlockId,
    ) -> Result<Option<GenericResponse<BlockHeaderData>>, Error> {
        let mut path = self.server.clone();

        path.path_segments_mut()
            .expect("path is base")
            .push("beacon")
            .push("headers")
            .push(&block_id.to_string());

        self.get_opt(path).await
    }

    /// `POST beacon/blocks`
    ///
    /// Returns `Ok(None)` on a 404 error.
    pub async fn post_beacon_blocks<T: EthSpec>(
        &self,
        block: SignedBeaconBlock<T>,
    ) -> Result<(), Error> {
        let mut path = self.server.clone();

        path.path_segments_mut()
            .expect("path is base")
            .push("beacon")
            .push("blocks");

        self.post(path, &block).await?;

        Ok(())
    }

    /// `GET beacon/blocks`
    ///
    /// Returns `Ok(None)` on a 404 error.
    pub async fn get_beacon_blocks<T: EthSpec>(
        &self,
        block_id: BlockId,
    ) -> Result<Option<GenericResponse<SignedBeaconBlock<T>>>, Error> {
        let mut path = self.server.clone();

        path.path_segments_mut()
            .expect("path is base")
            .push("beacon")
            .push("blocks")
            .push(&block_id.to_string());

        self.get_opt(path).await
    }

    /// `GET beacon/blocks/{block_id}/root`
    ///
    /// Returns `Ok(None)` on a 404 error.
    pub async fn get_beacon_blocks_root(
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

    /// `GET beacon/blocks/{block_id}/attestations`
    ///
    /// Returns `Ok(None)` on a 404 error.
    pub async fn get_beacon_blocks_attestations<T: EthSpec>(
        &self,
        block_id: BlockId,
    ) -> Result<Option<GenericResponse<Vec<Attestation<T>>>>, Error> {
        let mut path = self.server.clone();

        path.path_segments_mut()
            .expect("path is base")
            .push("beacon")
            .push("blocks")
            .push(&block_id.to_string())
            .push("attestations");

        self.get_opt(path).await
    }

    /// `POST beacon/pool/attestations`
    pub async fn post_beacon_pool_attestations<T: EthSpec>(
        &self,
        attestation: &Attestation<T>,
    ) -> Result<(), Error> {
        let mut path = self.server.clone();

        path.path_segments_mut()
            .expect("path is base")
            .push("beacon")
            .push("pool")
            .push("attestations");

        self.post(path, attestation).await?;

        Ok(())
    }

    /// `GET beacon/pool/attestations`
    pub async fn get_beacon_pool_attestations<T: EthSpec>(
        &self,
    ) -> Result<GenericResponse<Vec<Attestation<T>>>, Error> {
        let mut path = self.server.clone();

        path.path_segments_mut()
            .expect("path is base")
            .push("beacon")
            .push("pool")
            .push("attestations");

        self.get(path).await
    }

    /// `POST beacon/pool/attester_slashings`
    pub async fn post_beacon_pool_attester_slashings<T: EthSpec>(
        &self,
        slashing: &AttesterSlashing<T>,
    ) -> Result<(), Error> {
        let mut path = self.server.clone();

        path.path_segments_mut()
            .expect("path is base")
            .push("beacon")
            .push("pool")
            .push("attester_slashings");

        self.post(path, slashing).await?;

        Ok(())
    }

    /// `GET beacon/pool/attester_slashings`
    pub async fn get_beacon_pool_attester_slashings<T: EthSpec>(
        &self,
    ) -> Result<GenericResponse<Vec<AttesterSlashing<T>>>, Error> {
        let mut path = self.server.clone();

        path.path_segments_mut()
            .expect("path is base")
            .push("beacon")
            .push("pool")
            .push("attester_slashings");

        self.get(path).await
    }

    /// `POST beacon/pool/proposer_slashings`
    pub async fn post_beacon_pool_proposer_slashings(
        &self,
        slashing: &ProposerSlashing,
    ) -> Result<(), Error> {
        let mut path = self.server.clone();

        path.path_segments_mut()
            .expect("path is base")
            .push("beacon")
            .push("pool")
            .push("proposer_slashings");

        self.post(path, slashing).await?;

        Ok(())
    }

    /// `GET beacon/pool/proposer_slashings`
    pub async fn get_beacon_pool_proposer_slashings(
        &self,
    ) -> Result<GenericResponse<Vec<ProposerSlashing>>, Error> {
        let mut path = self.server.clone();

        path.path_segments_mut()
            .expect("path is base")
            .push("beacon")
            .push("pool")
            .push("proposer_slashings");

        self.get(path).await
    }

    /// `POST beacon/pool/voluntary_exits`
    pub async fn post_beacon_pool_voluntary_exits(
        &self,
        exit: &SignedVoluntaryExit,
    ) -> Result<(), Error> {
        let mut path = self.server.clone();

        path.path_segments_mut()
            .expect("path is base")
            .push("beacon")
            .push("pool")
            .push("voluntary_exits");

        self.post(path, exit).await?;

        Ok(())
    }

    /// `GET beacon/pool/voluntary_exits`
    pub async fn get_beacon_pool_voluntary_exits(
        &self,
    ) -> Result<GenericResponse<Vec<SignedVoluntaryExit>>, Error> {
        let mut path = self.server.clone();

        path.path_segments_mut()
            .expect("path is base")
            .push("beacon")
            .push("pool")
            .push("voluntary_exits");

        self.get(path).await
    }
}

/// Returns `Ok(response)` if the response is a `200 OK` response. Otherwise, creates an
/// appropriate error message.
async fn ok_or_error(response: Response) -> Result<Response, Error> {
    let status = response.status();

    if status == StatusCode::OK {
        Ok(response)
    } else {
        if let Some(message) = response.json().await.ok() {
            Err(Error::ServerMessage(message))
        } else {
            Err(Error::StatusCode(status))
        }
    }
}
