use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBootVolumeBackupRequest {
    /// The OCID of the boot volume backup.
    pub boot_volume_backup_id: String,
}

/// Required fields for GetBootVolumeBackupRequest
pub struct GetBootVolumeBackupRequestRequired {
    /// The OCID of the boot volume backup.
    pub boot_volume_backup_id: String,
}

impl GetBootVolumeBackupRequest {
    /// Create a new GetBootVolumeBackupRequest with required fields
    pub fn new(required: GetBootVolumeBackupRequestRequired) -> Self {
        Self {
            boot_volume_backup_id: required.boot_volume_backup_id,
        }
    }

    /// Set boot_volume_backup_id
    pub fn set_boot_volume_backup_id(mut self, value: String) -> Self {
        self.boot_volume_backup_id = value;
        self
    }
}
