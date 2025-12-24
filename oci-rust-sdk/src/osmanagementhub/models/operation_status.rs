use serde::{Deserialize, Serialize};

/// Possible operation statuses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationStatus {
    #[serde(rename = "WAITING")]
    Waiting,

    #[serde(rename = "ACCEPTED")]
    Accepted,

    #[serde(rename = "IN_PROGRESS")]
    InProgress,

    #[serde(rename = "FAILED")]
    Failed,

    #[serde(rename = "SUCCEEDED")]
    Succeeded,

    #[serde(rename = "CANCELING")]
    Canceling,

    #[serde(rename = "CANCELED")]
    Canceled,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
