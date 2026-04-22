use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVolumeGroupBackupRequest {
    /// The Oracle Cloud ID (OCID) that uniquely identifies the volume group backup.
    pub volume_group_backup_id: String,
}

/// Required fields for GetVolumeGroupBackupRequest
pub struct GetVolumeGroupBackupRequestRequired {
    /// The Oracle Cloud ID (OCID) that uniquely identifies the volume group backup.
    pub volume_group_backup_id: String,
}

impl GetVolumeGroupBackupRequest {
    /// Create a new GetVolumeGroupBackupRequest with required fields
    pub fn new(required: GetVolumeGroupBackupRequestRequired) -> Self {
        Self {
            volume_group_backup_id: required.volume_group_backup_id,
        }
    }

    /// Set volume_group_backup_id
    pub fn set_volume_group_backup_id(mut self, value: String) -> Self {
        self.volume_group_backup_id = value;
        self
    }
}
