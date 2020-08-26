use serde::{Deserialize, Serialize};
use types::{serde_utils, Epoch, Validator};

/// The number of epochs between when a validator is eligible for activation and when they
/// *usually* enter the activation queue.
const EPOCHS_BEFORE_FINALITY: u64 = 3;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidatorData {
    #[serde(with = "serde_utils::quoted")]
    pub index: u64,
    #[serde(with = "serde_utils::quoted")]
    pub balance: u64,
    pub status: ValidatorStatus,
    pub validator: Validator,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ValidatorStatus {
    Unknown,
    WaitingForEligibility,
    WaitingForFinality(Epoch),
    WaitingInQueue,
    StandbyForActive(Epoch),
    Active,
    ActiveAwaitingExit(Epoch),
    Exited(Epoch),
    Withdrawable,
}

impl ValidatorStatus {
    pub fn from_validator(
        validator_opt: Option<&Validator>,
        epoch: Epoch,
        finalized_epoch: Epoch,
        far_future_epoch: Epoch,
    ) -> Self {
        if let Some(validator) = validator_opt {
            if validator.is_withdrawable_at(epoch) {
                ValidatorStatus::Withdrawable
            } else if validator.is_exited_at(epoch) {
                ValidatorStatus::Exited(validator.withdrawable_epoch)
            } else if validator.is_active_at(epoch) {
                if validator.exit_epoch < far_future_epoch {
                    ValidatorStatus::ActiveAwaitingExit(validator.exit_epoch)
                } else {
                    ValidatorStatus::Active
                }
            } else {
                if validator.activation_epoch < far_future_epoch {
                    ValidatorStatus::StandbyForActive(validator.activation_epoch)
                } else if validator.activation_eligibility_epoch < far_future_epoch {
                    if finalized_epoch < validator.activation_eligibility_epoch {
                        ValidatorStatus::WaitingForFinality(
                            validator.activation_eligibility_epoch + EPOCHS_BEFORE_FINALITY,
                        )
                    } else {
                        ValidatorStatus::WaitingInQueue
                    }
                } else {
                    ValidatorStatus::WaitingForEligibility
                }
            }
        } else {
            ValidatorStatus::Unknown
        }
    }
}
