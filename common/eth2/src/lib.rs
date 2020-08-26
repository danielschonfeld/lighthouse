pub mod types;

use self::types::{GenericResponse, RootData, StateId};
use reqwest::Error;

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

    pub async fn beacon_states_root(
        &self,
        state_id: StateId,
    ) -> Result<GenericResponse<RootData>, Error> {
        self.client
            .get(&format!(
                "{}/{}/beacon/states/{}/root",
                self.server, VERSION, state_id
            ))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
}
