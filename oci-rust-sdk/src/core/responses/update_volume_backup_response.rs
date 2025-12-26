use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVolumeBackupResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// The returned model.VolumeBackup instance.
    pub volume_backup: VolumeBackup,
}

/// Required fields for UpdateVolumeBackupResponse
pub struct UpdateVolumeBackupResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// The returned model.VolumeBackup instance.
    pub volume_backup: VolumeBackup,
}

impl UpdateVolumeBackupResponse {
    /// Create a new UpdateVolumeBackupResponse with required fields
    pub fn new(required: UpdateVolumeBackupResponseRequired) -> Self {
        Self {
            etag: required.etag,

            volume_backup: required.volume_backup,
        }
    }

    /// Set etag
    pub fn set_etag(mut self, value: String) -> Self {
        self.etag = value;
        self
    }

    /// Set volume_backup
    pub fn set_volume_backup(mut self, value: VolumeBackup) -> Self {
        self.volume_backup = value;
        self
    }
}
