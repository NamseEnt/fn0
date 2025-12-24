use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies the source boot volume.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootVolumeSourceFromBootVolumeDetails {
    /// The OCID of the boot volume.
    pub id: String,

    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for BootVolumeSourceFromBootVolumeDetails
pub struct BootVolumeSourceFromBootVolumeDetailsRequired {
    /// The OCID of the boot volume.
    pub id: String,

    pub r#type: String,
}

impl BootVolumeSourceFromBootVolumeDetails {
    /// Create a new BootVolumeSourceFromBootVolumeDetails with required fields
    pub fn new(required: BootVolumeSourceFromBootVolumeDetailsRequired) -> Self {
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
