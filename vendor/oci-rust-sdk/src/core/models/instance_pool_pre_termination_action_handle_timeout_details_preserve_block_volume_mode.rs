use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstancePoolPreTerminationActionHandleTimeoutDetailsPreserveBlockVolumeMode {
    #[serde(rename = "PRESERVE_ALWAYS")]
    PreserveAlways,

    #[serde(rename = "PRESERVE_ON_TIMEOUT")]
    PreserveOnTimeout,

    #[serde(rename = "DELETE_ALWAYS")]
    DeleteAlways,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
