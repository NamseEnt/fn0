use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies the source for a volume group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeGroupSourceDetails {
    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for VolumeGroupSourceDetails
pub struct VolumeGroupSourceDetailsRequired {
    pub r#type: String,
}

impl VolumeGroupSourceDetails {
    /// Create a new VolumeGroupSourceDetails with required fields
    pub fn new(required: VolumeGroupSourceDetailsRequired) -> Self {
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
