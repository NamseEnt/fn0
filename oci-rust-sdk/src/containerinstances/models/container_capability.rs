use serde::{Deserialize, Serialize};

/// Additional configurable container capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerCapability {
    #[serde(rename = "CAP_NET_ADMIN")]
    CapNetAdmin,

    #[serde(rename = "CAP_NET_RAW")]
    CapNetRaw,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
