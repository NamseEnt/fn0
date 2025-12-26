use serde::{Deserialize, Serialize};

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
