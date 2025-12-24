use serde::{Deserialize, Serialize};

/// Severity of the security advisory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdvisorySeverity {
    #[serde(rename = "LOW")]
    Low,

    #[serde(rename = "MODERATE")]
    Moderate,

    #[serde(rename = "IMPORTANT")]
    Important,

    #[serde(rename = "CRITICAL")]
    Critical,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
