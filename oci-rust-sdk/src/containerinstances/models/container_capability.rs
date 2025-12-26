use serde::{Deserialize, Serialize};
use std::fmt;

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

impl fmt::Display for ContainerCapability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CapNetAdmin => write!(f, "CAP_NET_ADMIN"),

            Self::CapNetRaw => write!(f, "CAP_NET_RAW"),

            Self::UnknownValue => write!(f, "UNKNOWN"),
        }
    }
}
