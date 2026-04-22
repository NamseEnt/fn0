use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootVolumeSourceDetails {
    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for BootVolumeSourceDetails
pub struct BootVolumeSourceDetailsRequired {
    pub r#type: String,
}

impl BootVolumeSourceDetails {
    /// Create a new BootVolumeSourceDetails with required fields
    pub fn new(required: BootVolumeSourceDetailsRequired) -> Self {
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
