use serde_derive::{Deserialize, Serialize};
use types::{Epoch, Hash256, PublicKey, Slot};

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum InterchangeFormat {
    Minimal,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InterchangeMetadata {
    pub interchange_format: InterchangeFormat,
    #[serde(with = "types::serde_utils::quoted")]
    pub interchange_format_version: u64,
    pub genesis_validators_root: Hash256,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MinimalInterchangeData {
    pub pubkey: PublicKey,
    pub last_signed_block_slot: Option<Slot>,
    pub last_signed_attestation_source_epoch: Option<Epoch>,
    pub last_signed_attestation_target_epoch: Option<Epoch>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompleteInterchangeData {
    pub pubkey: PublicKey,
    pub signed_blocks: Vec<SignedBlock>,
    pub signed_attestations: Vec<SignedAttestation>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SignedBlock {
    pub slot: Slot,
    pub signing_root: Option<Hash256>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SignedAttestation {
    pub source_epoch: Epoch,
    pub target_epoch: Epoch,
    pub signing_root: Option<Hash256>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum InterchangeData {
    Minimal(Vec<MinimalInterchangeData>),
    Complete(Vec<CompleteInterchangeData>),
}

/// Temporary struct used during parsing.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PreInterchange {
    metadata: InterchangeMetadata,
    data: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Interchange {
    pub metadata: InterchangeMetadata,
    pub data: InterchangeData,
}

impl Interchange {
    pub fn from_json_str(json: &str) -> Result<Self, serde_json::Error> {
        let pre_interchange = serde_json::from_str(json)?;
        Self::from_pre_interchange(pre_interchange)
    }

    pub fn from_json_reader(reader: impl std::io::Read) -> Result<Self, serde_json::Error> {
        let pre_interchange = serde_json::from_reader(reader)?;
        Self::from_pre_interchange(pre_interchange)
    }

    pub fn write_to(&self, writer: impl std::io::Write) -> Result<(), serde_json::Error> {
        serde_json::to_writer(writer, self)
    }

    fn from_pre_interchange(pre_interchange: PreInterchange) -> Result<Self, serde_json::Error> {
        let metadata = pre_interchange.metadata;
        let data = match metadata.interchange_format {
            InterchangeFormat::Minimal => {
                InterchangeData::Minimal(serde_json::from_value(pre_interchange.data)?)
            }
            InterchangeFormat::Complete => {
                InterchangeData::Complete(serde_json::from_value(pre_interchange.data)?)
            }
        };
        Ok(Interchange { metadata, data })
    }
}
