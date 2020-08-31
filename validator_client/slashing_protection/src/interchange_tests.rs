#![cfg(test)]

use crate::interchange::{
    Interchange, InterchangeData, InterchangeFormat, InterchangeMetadata, MinimalInterchangeData,
};
use crate::test_utils::pubkey;
use crate::{InvalidBlock, NotSafe, SlashingDatabase, SUPPORTED_INTERCHANGE_FORMAT_VERSION};
use tempfile::tempdir;
use types::{Epoch, Hash256, Slot};

#[test]
fn import_minimal_single_basic1() {
    let data = vec![MinimalInterchangeData {
        pubkey: pubkey(0),
        last_signed_block_slot: Slot::new(10),
        last_signed_attestation_source_epoch: Epoch::new(1),
        last_signed_attestation_target_epoch: Epoch::new(2),
    }];
    import_minimal_test(data.clone());
    double_import_minimal(data);
}

#[test]
fn import_minimal_single_basic2() {
    let data = vec![MinimalInterchangeData {
        pubkey: pubkey(0),
        last_signed_block_slot: Slot::new(15670),
        last_signed_attestation_source_epoch: Epoch::new(200),
        last_signed_attestation_target_epoch: Epoch::new(305),
    }];
    import_minimal_test(data.clone());
    double_import_minimal(data);
}

#[test]
fn import_minimal_single_all_zero() {
    let data = vec![MinimalInterchangeData {
        pubkey: pubkey(0),
        last_signed_block_slot: Slot::new(0),
        last_signed_attestation_source_epoch: Epoch::new(0),
        last_signed_attestation_target_epoch: Epoch::new(0),
    }];
    import_minimal_test(data.clone());
    double_import_minimal(data);
}

#[test]
fn import_minimal_single_equal_epoch() {
    let data = vec![MinimalInterchangeData {
        pubkey: pubkey(0),
        last_signed_block_slot: Slot::new(0),
        last_signed_attestation_source_epoch: Epoch::new(10),
        last_signed_attestation_target_epoch: Epoch::new(10),
    }];
    import_minimal_test(data.clone());
    double_import_minimal(data);
}

#[test]
fn import_minimal_single_big() {
    let data = vec![MinimalInterchangeData {
        pubkey: pubkey(0),
        last_signed_block_slot: Slot::new(1_048_576),
        last_signed_attestation_source_epoch: Epoch::new(32_767),
        last_signed_attestation_target_epoch: Epoch::new(32_768),
    }];
    // Don't verify, because it takes too long, just check we're able to import within
    // a reasonable time.
    double_import_minimal(data);
}

fn import_minimal_test(data: Vec<MinimalInterchangeData>) {
    let dir = tempdir().unwrap();
    let slashing_db_file = dir.path().join("slashing_protection.sqlite");
    let slashing_db = SlashingDatabase::create(&slashing_db_file).unwrap();

    let genesis_validators_root = Hash256::from_low_u64_be(66);
    let interchange = Interchange {
        metadata: InterchangeMetadata {
            interchange_format: InterchangeFormat::Minimal,
            interchange_format_version: SUPPORTED_INTERCHANGE_FORMAT_VERSION,
            genesis_validators_root,
        },
        data: InterchangeData::Minimal(data.clone()),
    };

    slashing_db
        .import_interchange_info(&interchange, genesis_validators_root)
        .unwrap();

    for validator in data {
        // Blocks with slots less than or equal to the last signed block slot should be rejected.
        for slot in 0..=validator.last_signed_block_slot.as_u64() {
            let res = slashing_db.check_and_insert_block_signing_root(
                &validator.pubkey,
                Slot::new(slot),
                Hash256::from_low_u64_be(slot + 1),
            );
            assert!(matches!(
                res.unwrap_err(),
                NotSafe::InvalidBlock(InvalidBlock::SlotViolatesLowerBound { .. })
            ));
        }

        // A block at the next slot is permissible.
        slashing_db
            .check_and_insert_block_signing_root(
                &validator.pubkey,
                validator.last_signed_block_slot + 1,
                Hash256::from_low_u64_be(1),
            )
            .unwrap();

        // Attestations at all targets between 0 and the target limit (inclusive) should be barred.
        for epoch in 0..=validator.last_signed_attestation_target_epoch.as_u64() {
            let target = Epoch::new(epoch);
            let source = Epoch::new(epoch.saturating_sub(1));
            let res = slashing_db.check_and_insert_attestation_signing_root(
                &validator.pubkey,
                source,
                target,
                Hash256::from_low_u64_be(epoch + 1),
            );
            assert!(matches!(res.unwrap_err(), NotSafe::InvalidAttestation(_)));
        }

        // An attestation that surrounds max source and max target should be barred.
        if validator.last_signed_attestation_source_epoch
            != validator.last_signed_attestation_target_epoch
        {
            let err = slashing_db
                .check_and_insert_attestation_signing_root(
                    &validator.pubkey,
                    validator.last_signed_attestation_source_epoch - 1,
                    validator.last_signed_attestation_target_epoch + 1,
                    Hash256::from_low_u64_be(1),
                )
                .unwrap_err();
            assert!(matches!(err, NotSafe::InvalidAttestation(_)));
        }

        // An attestation from the max source to the next epoch is OK.
        slashing_db
            .check_and_insert_attestation_signing_root(
                &validator.pubkey,
                validator.last_signed_attestation_source_epoch,
                validator.last_signed_attestation_target_epoch + 1,
                Hash256::from_low_u64_be(1),
            )
            .unwrap();
    }
}

// Importing the same minimal interchange data twice should succeed.
fn double_import_minimal(data: Vec<MinimalInterchangeData>) {
    let dir = tempdir().unwrap();
    let slashing_db_file = dir.path().join("slashing_protection.sqlite");
    let slashing_db = SlashingDatabase::create(&slashing_db_file).unwrap();

    let genesis_validators_root = Hash256::from_low_u64_be(66);
    let interchange = Interchange {
        metadata: InterchangeMetadata {
            interchange_format: InterchangeFormat::Minimal,
            interchange_format_version: SUPPORTED_INTERCHANGE_FORMAT_VERSION,
            genesis_validators_root,
        },
        data: InterchangeData::Minimal(data.clone()),
    };

    slashing_db
        .import_interchange_info(&interchange, genesis_validators_root)
        .unwrap();
    slashing_db
        .import_interchange_info(&interchange, genesis_validators_root)
        .unwrap();
}
