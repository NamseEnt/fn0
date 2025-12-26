use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies the volume source details for a new Block volume. The volume source is either another Block volume in the same Availability Domain or a Block volume backup. This is an optional field. If not specified or set to null, the new Block volume will be empty. When specified, the new Block volume will contain data from the source volume or backup.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeSourceDetails {
    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for VolumeSourceDetails
pub struct VolumeSourceDetailsRequired {
    pub r#type: String,
}

impl VolumeSourceDetails {
    /// Create a new VolumeSourceDetails with required fields
    pub fn new(required: VolumeSourceDetailsRequired) -> Self {
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
