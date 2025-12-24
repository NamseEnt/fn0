use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A volume represents a directory with data that is accessible across multiple containers in a container instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateContainerVolumeDetails {
    /// The name of the volume. This must be unique within a single container instance.
    pub name: String,

    pub volume_type: String,
}

/// Required fields for CreateContainerVolumeDetails
pub struct CreateContainerVolumeDetailsRequired {
    /// The name of the volume. This must be unique within a single container instance.
    pub name: String,

    pub volume_type: String,
}

impl CreateContainerVolumeDetails {
    /// Create a new CreateContainerVolumeDetails with required fields
    pub fn new(required: CreateContainerVolumeDetailsRequired) -> Self {
        Self {
            name: required.name,

            volume_type: required.volume_type,
        }
    }

    /// Set name
    pub fn set_name(mut self, value: String) -> Self {
        self.name = value;
        self
    }

    /// Set volume_type
    pub fn set_volume_type(mut self, value: String) -> Self {
        self.volume_type = value;
        self
    }
}
