use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachServiceDeterminedVolumeDetails {
    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for AttachServiceDeterminedVolumeDetails
pub struct AttachServiceDeterminedVolumeDetailsRequired {
    pub r#type: String,
}

impl AttachServiceDeterminedVolumeDetails {
    /// Create a new AttachServiceDeterminedVolumeDetails with required fields
    pub fn new(required: AttachServiceDeterminedVolumeDetailsRequired) -> Self {
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
