use std::cmp;
use types::{Epoch, Slot};

#[derive(Debug, Clone, Copy, Default)]
pub struct LowerBound {
    pub block_proposal_slot: Slot,
    pub attestation_source_epoch: Epoch,
    pub attestation_target_epoch: Epoch,
}

impl LowerBound {
    pub fn update(self, other: Self) -> Self {
        Self {
            block_proposal_slot: cmp::max(self.block_proposal_slot, other.block_proposal_slot),
            attestation_source_epoch: cmp::max(
                self.attestation_source_epoch,
                other.attestation_source_epoch,
            ),
            attestation_target_epoch: cmp::max(
                self.attestation_target_epoch,
                other.attestation_target_epoch,
            ),
        }
    }

    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        let block_proposal_slot = row.get(0)?;
        let attestation_source_epoch = row.get(1)?;
        let attestation_target_epoch = row.get(2)?;
        Ok(Self {
            block_proposal_slot,
            attestation_source_epoch,
            attestation_target_epoch,
        })
    }
}
