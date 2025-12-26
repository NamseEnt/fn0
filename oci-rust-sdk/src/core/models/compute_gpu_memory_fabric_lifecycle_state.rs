use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComputeGpuMemoryFabricLifecycleState {
    #[serde(rename = "AVAILABLE")]
    Available,

    #[serde(rename = "OCCUPIED")]
    Occupied,

    #[serde(rename = "PROVISIONING")]
    Provisioning,

    #[serde(rename = "DEGRADED")]
    Degraded,

    #[serde(rename = "UNAVAILABLE")]
    Unavailable,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
