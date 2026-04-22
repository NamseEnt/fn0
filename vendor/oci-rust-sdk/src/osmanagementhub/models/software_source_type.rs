use serde::{Deserialize, Serialize};

/// Type of software source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoftwareSourceType {
    #[serde(rename = "VENDOR")]
    Vendor,

    #[serde(rename = "CUSTOM")]
    Custom,

    #[serde(rename = "VERSIONED")]
    Versioned,

    #[serde(rename = "PRIVATE")]
    Private,

    #[serde(rename = "THIRD_PARTY")]
    ThirdParty,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
