use serde::{Deserialize, Serialize};

/// Architecture type of a software package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoftwarePackageArchitecture {
    #[serde(rename = "I386")]
    I386,

    #[serde(rename = "I686")]
    I686,

    #[serde(rename = "AARCH64")]
    Aarch64,

    #[serde(rename = "X86_64")]
    X8664,

    #[serde(rename = "SRC")]
    Src,

    #[serde(rename = "NOARCH")]
    Noarch,

    #[serde(rename = "OTHER")]
    Other,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
