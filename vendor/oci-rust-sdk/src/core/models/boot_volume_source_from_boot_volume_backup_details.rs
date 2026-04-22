use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies the boot volume backup.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootVolumeSourceFromBootVolumeBackupDetails {
    /// The OCID of the boot volume backup.
    pub id: String,

    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for BootVolumeSourceFromBootVolumeBackupDetails
pub struct BootVolumeSourceFromBootVolumeBackupDetailsRequired {
    /// The OCID of the boot volume backup.
    pub id: String,

    pub r#type: String,
}

impl BootVolumeSourceFromBootVolumeBackupDetails {
    /// Create a new BootVolumeSourceFromBootVolumeBackupDetails with required fields
    pub fn new(required: BootVolumeSourceFromBootVolumeBackupDetailsRequired) -> Self {
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
