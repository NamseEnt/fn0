use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComputeGpuMemoryFabricSummaryFirmwareUpdateState {
    #[serde(rename = "WILL_UPDATE")]
    WillUpdate,

    #[serde(rename = "NO_UPDATE")]
    NoUpdate,

    #[serde(rename = "SKIP_RECYCLE_ENABLED")]
    SkipRecycleEnabled,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
