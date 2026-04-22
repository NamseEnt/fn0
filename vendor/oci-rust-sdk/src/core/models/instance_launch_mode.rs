use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstanceLaunchMode {
    #[serde(rename = "NATIVE")]
    Native,

    #[serde(rename = "EMULATED")]
    Emulated,

    #[serde(rename = "PARAVIRTUALIZED")]
    Paravirtualized,

    #[serde(rename = "CUSTOM")]
    Custom,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
