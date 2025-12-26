use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Define a volume that will be created and attached or attached to an instance on creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchCreateVolumeDetails {
    pub volume_creation_type: String,
}

/// Required fields for LaunchCreateVolumeDetails
pub struct LaunchCreateVolumeDetailsRequired {
    pub volume_creation_type: String,
}

impl LaunchCreateVolumeDetails {
    /// Create a new LaunchCreateVolumeDetails with required fields
    pub fn new(required: LaunchCreateVolumeDetailsRequired) -> Self {
        Self {
            volume_creation_type: required.volume_creation_type,
        }
    }

    /// Set volume_creation_type
    pub fn set_volume_creation_type(mut self, value: String) -> Self {
        self.volume_creation_type = value;
        self
    }
}
