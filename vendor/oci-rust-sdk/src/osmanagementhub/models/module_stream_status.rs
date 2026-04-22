use serde::{Deserialize, Serialize};

/// The status of a module stream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleStreamStatus {
    #[serde(rename = "ENABLED")]
    Enabled,

    #[serde(rename = "DISABLED")]
    Disabled,

    #[serde(rename = "ACTIVE")]
    Active,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
