use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateInstanceAvailabilityConfigDetailsRecoveryAction {
    #[serde(rename = "RESTORE_INSTANCE")]
    RestoreInstance,

    #[serde(rename = "STOP_INSTANCE")]
    StopInstance,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
