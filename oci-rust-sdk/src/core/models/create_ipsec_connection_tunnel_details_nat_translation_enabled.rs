use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CreateIPSecConnectionTunnelDetailsNatTranslationEnabled {
    #[serde(rename = "ENABLED")]
    Enabled,

    #[serde(rename = "DISABLED")]
    Disabled,

    #[serde(rename = "AUTO")]
    Auto,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
