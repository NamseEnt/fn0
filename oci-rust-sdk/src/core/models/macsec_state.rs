use serde::{Deserialize, Serialize};

/// Indicates whether or not MACsec is enabled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MacsecState {
    #[serde(rename = "ENABLED")]
    Enabled,

    #[serde(rename = "DISABLED")]
    Disabled,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
