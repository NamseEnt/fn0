use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachEmulatedVolumeDetails {
    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for AttachEmulatedVolumeDetails
pub struct AttachEmulatedVolumeDetailsRequired {
    pub r#type: String,
}

impl AttachEmulatedVolumeDetails {
    /// Create a new AttachEmulatedVolumeDetails with required fields
    pub fn new(required: AttachEmulatedVolumeDetailsRequired) -> Self {
        Self {
            r#type: required.r#type,
        }
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
