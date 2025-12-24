use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies the volume group backup to restore from.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeGroupSourceFromVolumeGroupBackupDetails {
    /// The OCID of the volume group backup to restore from.
    pub volume_group_backup_id: String,

    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for VolumeGroupSourceFromVolumeGroupBackupDetails
pub struct VolumeGroupSourceFromVolumeGroupBackupDetailsRequired {
    /// The OCID of the volume group backup to restore from.
    pub volume_group_backup_id: String,

    pub r#type: String,
}

impl VolumeGroupSourceFromVolumeGroupBackupDetails {
    /// Create a new VolumeGroupSourceFromVolumeGroupBackupDetails with required fields
    pub fn new(required: VolumeGroupSourceFromVolumeGroupBackupDetailsRequired) -> Self {
        Self {
            volume_group_backup_id: required.volume_group_backup_id,

            r#type: required.r#type,
        }
    }

    /// Set volume_group_backup_id
    pub fn set_volume_group_backup_id(mut self, value: String) -> Self {
        self.volume_group_backup_id = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
