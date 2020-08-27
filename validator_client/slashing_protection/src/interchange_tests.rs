#![cfg(test)]

use crate::interchange::{
    Interchange, InterchangeData, InterchangeFormat, InterchangeMetadata, MinimalInterchangeData,
};
use crate::test_utils::pubkey;
use crate::{NotSafe, SlashingDatabase};
use tempfile::tempdir;
use types::{Epoch, Hash256, Slot};

#[test]
fn import_minimal_single() {
    import_minimal_test(vec![MinimalInterchangeData {
        pubkey: pubkey(0),
        last_signed_block_slot: Slot::new(10_000),
        last_signed_attestation_source_epoch: Epoch::new(1),
        last_signed_attestation_target_epoch: Epoch::new(2),
    }])
}

fn import_minimal_test(data: Vec<MinimalInterchangeData>) {
    let dir = tempdir().unwrap();
    let slashing_db_file = dir.path().join("slashing_protection.sqlite");
    let slashing_db = SlashingDatabase::create(&slashing_db_file).unwrap();

    let genesis_validators_root = Hash256::from_low_u64_be(66);
    let interchange = Interchange {
        metadata: InterchangeMetadata {
            interchange_format: InterchangeFormat::Minimal,
            interchange_format_version: 1,
            genesis_validators_root,
        },
        data: InterchangeData::Minimal(data.clone()),
    };

    slashing_db
        .import_interchange_info(&interchange, genesis_validators_root)
        .unwrap();

    for validator in data {
        for slot in 0..validator.last_signed_block_slot.as_u64() {
            let res = slashing_db.check_and_insert_block_signing_root(
                &validator.pubkey,
                Slot::new(slot),
                Hash256::from_low_u64_be(slot + 1),
            );
            assert!(matches!(res.unwrap_err(), NotSafe::InvalidBlock(_)));
        }
    }
}
