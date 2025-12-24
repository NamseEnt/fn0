use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VirtualCircuitPublicPrefixVerificationState {
    #[serde(rename = "IN_PROGRESS")]
    InProgress,

    #[serde(rename = "COMPLETED")]
    Completed,

    #[serde(rename = "FAILED")]
    Failed,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
