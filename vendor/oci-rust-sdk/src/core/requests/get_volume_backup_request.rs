use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVolumeBackupRequest {
    /// The OCID of the volume backup.
    pub volume_backup_id: String,
}

/// Required fields for GetVolumeBackupRequest
pub struct GetVolumeBackupRequestRequired {
    /// The OCID of the volume backup.
    pub volume_backup_id: String,
}

impl GetVolumeBackupRequest {
    /// Create a new GetVolumeBackupRequest with required fields
    pub fn new(required: GetVolumeBackupRequestRequired) -> Self {
        Self {
            volume_backup_id: required.volume_backup_id,
        }
    }

    /// Set volume_backup_id
    pub fn set_volume_backup_id(mut self, value: String) -> Self {
        self.volume_backup_id = value;
        self
    }
}
