use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The empty directory volume of a container instance. You can create up to 64 EmptyDir per container instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerEmptyDirVolume {
    pub volume_type: String,

    /// The volume type of the empty directory, can be either File Storage or Memory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backing_store: Option<ContainerEmptyDirVolumeBackingStore>,
}

/// Required fields for ContainerEmptyDirVolume
pub struct ContainerEmptyDirVolumeRequired {
    pub volume_type: String,
}

impl ContainerEmptyDirVolume {
    /// Create a new ContainerEmptyDirVolume with required fields
    pub fn new(required: ContainerEmptyDirVolumeRequired) -> Self {
        Self {
            volume_type: required.volume_type,

            backing_store: None,
        }
    }

    /// Set backing_store
    pub fn set_backing_store(mut self, value: Option<ContainerEmptyDirVolumeBackingStore>) -> Self {
        self.backing_store = value;
        self
    }

    /// Set volume_type
    pub fn set_volume_type(mut self, value: String) -> Self {
        self.volume_type = value;
        self
    }

    /// Set backing_store (unwraps Option)
    pub fn with_backing_store(mut self, value: ContainerEmptyDirVolumeBackingStore) -> Self {
        self.backing_store = Some(value);
        self
    }
}
