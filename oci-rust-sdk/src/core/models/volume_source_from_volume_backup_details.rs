use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies the volume backup.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeSourceFromVolumeBackupDetails {
    /// The OCID of the volume backup.
    pub id: String,

    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for VolumeSourceFromVolumeBackupDetails
pub struct VolumeSourceFromVolumeBackupDetailsRequired {
    /// The OCID of the volume backup.
    pub id: String,

    pub r#type: String,
}

impl VolumeSourceFromVolumeBackupDetails {
    /// Create a new VolumeSourceFromVolumeBackupDetails with required fields
    pub fn new(required: VolumeSourceFromVolumeBackupDetailsRequired) -> Self {
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
