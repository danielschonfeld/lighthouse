use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Quoted<T>
where
    T: From<u64> + Into<u64> + Copy,
{
    #[serde(with = "types::serde_utils::only_quoted")]
    pub value: T,
}
