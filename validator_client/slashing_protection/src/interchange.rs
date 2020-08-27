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
    pub interchange_format_version: u16,
    pub genesis_validators_root: Hash256,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MinimalInterchangeData {
    pub pubkey: PublicKey,
    pub last_signed_block_slot: Slot,
    pub last_signed_attestation_source_epoch: Epoch,
    pub last_signed_attestation_target_epoch: Epoch,
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
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let pre_interchange = serde_json::from_str::<PreInterchange>(json)?;
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn minimal_example() {
        let interchange_json = r#"
            {
                "metadata": {
                    "interchange_format": "minimal",
                    "interchange_format_version": 1,
                    "genesis_validators_root": "0x04700007fabc8282644aed6d1c7c9e21d38a03a0c4ba193f3afe428824b3a673"
                },
                "data": [
                    {
                        "pubkey": "0xb845089a1457f811bfc000588fbb4e713669be8ce060ea6be3c6ece09afc3794106c91ca73acda5e5457122d58723bed",
                        "last_signed_block_slot": 89765,
                        "last_signed_attestation_source_epoch": 2990,
                        "last_signed_attestation_target_epoch": 3007
                    }
                ]
            }
        "#;
        let interchange = Interchange::from_json(interchange_json).unwrap();
        println!("{:#?}", interchange);
        println!("{}", serde_json::to_string_pretty(&interchange).unwrap());
    }
}
