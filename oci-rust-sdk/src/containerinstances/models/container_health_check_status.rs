use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerHealthCheckStatus {
    #[serde(rename = "HEALTHY")]
    Healthy,

    #[serde(rename = "UNHEALTHY")]
    Unhealthy,

    #[serde(rename = "UNKNOWN")]
    Unknown,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}

impl fmt::Display for ContainerHealthCheckStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Healthy => write!(f, "HEALTHY"),

            Self::Unhealthy => write!(f, "UNHEALTHY"),

            Self::Unknown => write!(f, "UNKNOWN"),

            Self::UnknownValue => write!(f, "UNKNOWN"),
        }
    }
}
