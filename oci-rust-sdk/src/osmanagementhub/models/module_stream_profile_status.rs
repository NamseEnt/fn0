use serde::{Deserialize, Serialize};

/// The status of a module stream profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleStreamProfileStatus {
    #[serde(rename = "INSTALLED")]
    Installed,

    #[serde(rename = "AVAILABLE")]
    Available,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
