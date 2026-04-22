use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies the source volume.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeSourceFromVolumeDetails {
    /// The OCID of the volume.
    pub id: String,

    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for VolumeSourceFromVolumeDetails
pub struct VolumeSourceFromVolumeDetailsRequired {
    /// The OCID of the volume.
    pub id: String,

    pub r#type: String,
}

impl VolumeSourceFromVolumeDetails {
    /// Create a new VolumeSourceFromVolumeDetails with required fields
    pub fn new(required: VolumeSourceFromVolumeDetailsRequired) -> Self {
        Self {
            id: required.id,

            r#type: required.r#type,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
