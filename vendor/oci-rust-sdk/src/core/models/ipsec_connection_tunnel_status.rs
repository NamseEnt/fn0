use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IPSecConnectionTunnelStatus {
    #[serde(rename = "UP")]
    Up,

    #[serde(rename = "DOWN")]
    Down,

    #[serde(rename = "DOWN_FOR_MAINTENANCE")]
    DownForMaintenance,

    #[serde(rename = "PARTIAL_UP")]
    PartialUp,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
