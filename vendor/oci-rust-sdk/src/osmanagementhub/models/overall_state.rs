use serde::{Deserialize, Serialize};

/// Overall state of the mirror.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverallState {
    #[serde(rename = "NORMAL")]
    Normal,

    #[serde(rename = "REGISTRATIONERROR")]
    Registrationerror,

    #[serde(rename = "SYNCING")]
    Syncing,

    #[serde(rename = "SYNCFAILED")]
    Syncfailed,

    #[serde(rename = "WARNING")]
    Warning,

    #[serde(rename = "ERROR")]
    Error,

    #[serde(rename = "UNAVAILABLE")]
    Unavailable,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
