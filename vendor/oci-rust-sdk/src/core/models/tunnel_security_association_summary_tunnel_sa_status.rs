use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TunnelSecurityAssociationSummaryTunnelSaStatus {
    #[serde(rename = "INITIATING")]
    Initiating,

    #[serde(rename = "LISTENING")]
    Listening,

    #[serde(rename = "UP")]
    Up,

    #[serde(rename = "DOWN")]
    Down,

    #[serde(rename = "ERROR")]
    Error,

    #[serde(rename = "UNKNOWN")]
    Unknown,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
