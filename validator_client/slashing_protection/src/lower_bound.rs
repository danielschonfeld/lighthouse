use std::cmp;
use types::{Epoch, Slot};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LowerBound {
    pub block_proposal_slot: Option<Slot>,
    pub attestation_source_epoch: Option<Epoch>,
    pub attestation_target_epoch: Option<Epoch>,
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn update_nones() {
        let all_none = LowerBound {
            block_proposal_slot: None,
            attestation_source_epoch: None,
            attestation_target_epoch: None,
        };
        assert_eq!(all_none, LowerBound::default());
        let all_some = LowerBound {
            block_proposal_slot: Some(Slot::new(10)),
            attestation_source_epoch: Some(Epoch::new(10)),
            attestation_target_epoch: Some(Epoch::new(10)),
        };
        assert_eq!(all_none.update(all_some), all_some);
        assert_eq!(all_some.update(all_none), all_some);
    }

    #[test]
    fn simple() {
        assert_eq!(
            LowerBound {
                block_proposal_slot: Some(Slot::new(1)),
                attestation_source_epoch: Some(Epoch::new(0)),
                attestation_target_epoch: Some(Epoch::new(100)),
            }
            .update(LowerBound {
                block_proposal_slot: Some(Slot::new(0)),
                attestation_source_epoch: Some(Epoch::new(96)),
                attestation_target_epoch: Some(Epoch::new(98)),
            }),
            LowerBound {
                block_proposal_slot: Some(Slot::new(1)),
                attestation_source_epoch: Some(Epoch::new(96)),
                attestation_target_epoch: Some(Epoch::new(100)),
            }
        );
    }
}
