use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrossConnectStatusLightLevelIndicator {
    #[serde(rename = "NO_LIGHT")]
    NoLight,

    #[serde(rename = "LOW_WARN")]
    LowWarn,

    #[serde(rename = "HIGH_WARN")]
    HighWarn,

    #[serde(rename = "BAD")]
    Bad,

    #[serde(rename = "GOOD")]
    Good,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
