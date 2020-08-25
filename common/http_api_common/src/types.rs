use serde::{Deserialize, Serialize};
use types::{Checkpoint, Hash256};

#[derive(Serialize)]
pub struct GenericResponse<T: Serialize> {
    pub data: T,
}

impl<T: Serialize> From<T> for GenericResponse<T> {
    fn from(data: T) -> Self {
        Self { data }
    }
}

#[derive(Serialize, Deserialize)]
pub struct RootData {
    pub root: Hash256,
}

impl From<Hash256> for RootData {
    fn from(root: Hash256) -> Self {
        Self { root }
    }
}

#[derive(Serialize, Deserialize)]
pub struct FinalityCheckpointsData {
    pub previous_justified: Checkpoint,
    pub current_justified: Checkpoint,
    pub finalized: Checkpoint,
}
