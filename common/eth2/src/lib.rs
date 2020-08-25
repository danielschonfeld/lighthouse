pub mod types;

use self::types::{GenericResponse, RootData, StateId};
use reqwest::Error;

pub struct BeaconNodeClient {
    client: reqwest::Client,
    server: String,
}

impl BeaconNodeClient {
    pub fn new(server: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            server: server.into(),
        }
    }

    pub async fn beacon_states_root(
        &self,
        state_id: StateId,
    ) -> Result<GenericResponse<RootData>, Error> {
        self.client
            .get(&format!("{}/beacon/states/{}/root", self.server, state_id))
            .send()
            .await?
            .json()
            .await
    }
}
