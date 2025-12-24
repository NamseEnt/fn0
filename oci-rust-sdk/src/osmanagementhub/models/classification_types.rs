use serde::{Deserialize, Serialize};

/// Type of software update.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClassificationTypes {
    #[serde(rename = "SECURITY")]
    Security,

    #[serde(rename = "BUGFIX")]
    Bugfix,

    #[serde(rename = "ENHANCEMENT")]
    Enhancement,

    #[serde(rename = "OTHER")]
    Other,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
