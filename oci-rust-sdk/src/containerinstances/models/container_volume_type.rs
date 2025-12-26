use serde::{Deserialize, Serialize};
use std::fmt;

/// Container Volume Type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerVolumeType {
    #[serde(rename = "EMPTYDIR")]
    Emptydir,

    #[serde(rename = "CONFIGFILE")]
    Configfile,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}

impl fmt::Display for ContainerVolumeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Emptydir => write!(f, "EMPTYDIR"),

            Self::Configfile => write!(f, "CONFIGFILE"),

            Self::UnknownValue => write!(f, "UNKNOWN"),
        }
    }
}
