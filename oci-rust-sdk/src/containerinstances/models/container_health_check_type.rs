use serde::{Deserialize, Serialize};
use std::fmt;

/// The container health check type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerHealthCheckType {
    #[serde(rename = "HTTP")]
    Http,

    #[serde(rename = "TCP")]
    Tcp,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}

impl fmt::Display for ContainerHealthCheckType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http => write!(f, "HTTP"),

            Self::Tcp => write!(f, "TCP"),

            Self::UnknownValue => write!(f, "UNKNOWN"),
        }
    }
}
