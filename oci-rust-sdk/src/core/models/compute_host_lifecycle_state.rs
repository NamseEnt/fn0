use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComputeHostLifecycleState {
    #[serde(rename = "AVAILABLE")]
    Available,

    #[serde(rename = "OCCUPIED")]
    Occupied,

    #[serde(rename = "PROVISIONING")]
    Provisioning,

    #[serde(rename = "REPAIR")]
    Repair,

    #[serde(rename = "UNAVAILABLE")]
    Unavailable,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
