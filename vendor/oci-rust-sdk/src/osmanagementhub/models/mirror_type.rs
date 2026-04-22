use serde::{Deserialize, Serialize};

/// Type of mirrored software source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MirrorType {
    #[serde(rename = "CUSTOM")]
    Custom,

    #[serde(rename = "VENDOR")]
    Vendor,

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
