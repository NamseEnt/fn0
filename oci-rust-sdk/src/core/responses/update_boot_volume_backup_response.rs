use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBootVolumeBackupResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// The returned model.BootVolumeBackup instance.
    pub boot_volume_backup: BootVolumeBackup,
}


/// Required fields for UpdateBootVolumeBackupResponse
pub struct UpdateBootVolumeBackupResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// The returned model.BootVolumeBackup instance.
    pub boot_volume_backup: BootVolumeBackup,
}

impl UpdateBootVolumeBackupResponse {
    /// Create a new UpdateBootVolumeBackupResponse with required fields
    pub fn new(required: UpdateBootVolumeBackupResponseRequired) -> Self {
        Self {
            etag: required.etag,

            boot_volume_backup: required.boot_volume_backup,
}
    }

    /// Set etag
    pub fn set_etag(mut self, value: String) -> Self {
        self.etag = value;
        self
    }

    /// Set boot_volume_backup
    pub fn set_boot_volume_backup(mut self, value: BootVolumeBackup) -> Self {
        self.boot_volume_backup = value;
        self
    }
}


