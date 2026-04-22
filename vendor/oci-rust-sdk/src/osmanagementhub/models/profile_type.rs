use serde::{Deserialize, Serialize};

/// Registration profile type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileType {
    #[serde(rename = "SOFTWARESOURCE")]
    Softwaresource,

    #[serde(rename = "GROUP")]
    Group,

    #[serde(rename = "LIFECYCLE")]
    Lifecycle,

    #[serde(rename = "STATION")]
    Station,

    #[serde(rename = "WINDOWS_STANDALONE")]
    WindowsStandalone,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
