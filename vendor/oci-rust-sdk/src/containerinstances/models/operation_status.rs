use serde::{Deserialize, Serialize};
use std::fmt;

/// Possible operation status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationStatus {
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

impl fmt::Display for OperationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Accepted => write!(f, "ACCEPTED"),

            Self::InProgress => write!(f, "IN_PROGRESS"),

            Self::Failed => write!(f, "FAILED"),

            Self::Succeeded => write!(f, "SUCCEEDED"),

            Self::Canceling => write!(f, "CANCELING"),

            Self::Canceled => write!(f, "CANCELED"),

            Self::UnknownValue => write!(f, "UNKNOWN"),
        }
    }
}
