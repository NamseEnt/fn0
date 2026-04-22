use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerEmptyDirVolumeBackingStore {
    #[serde(rename = "EPHEMERAL_STORAGE")]
    EphemeralStorage,

    #[serde(rename = "MEMORY")]
    Memory,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}

impl fmt::Display for ContainerEmptyDirVolumeBackingStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EphemeralStorage => write!(f, "EPHEMERAL_STORAGE"),

            Self::Memory => write!(f, "MEMORY"),

            Self::UnknownValue => write!(f, "UNKNOWN"),
        }
    }
}
