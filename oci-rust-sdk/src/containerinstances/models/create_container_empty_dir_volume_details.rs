use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The empty directory for the container instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateContainerEmptyDirVolumeDetails {
    pub volume_type: String,

    /// The volume type of the empty directory, can be either File Storage or Memory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backing_store: Option<String>,
}

/// Required fields for CreateContainerEmptyDirVolumeDetails
pub struct CreateContainerEmptyDirVolumeDetailsRequired {
    pub volume_type: String,
}

impl CreateContainerEmptyDirVolumeDetails {
    /// Create a new CreateContainerEmptyDirVolumeDetails with required fields
    pub fn new(required: CreateContainerEmptyDirVolumeDetailsRequired) -> Self {
        Self {
            volume_type: required.volume_type,

            backing_store: None,
        }
    }

    /// Set backing_store
    pub fn set_backing_store(mut self, value: Option<String>) -> Self {
        self.backing_store = value;
        self
    }

    /// Set volume_type
    pub fn set_volume_type(mut self, value: String) -> Self {
        self.volume_type = value;
        self
    }

    /// Set backing_store (unwraps Option)
    pub fn with_backing_store(mut self, value: impl Into<String>) -> Self {
        self.backing_store = Some(value.into());
        self
    }
}
