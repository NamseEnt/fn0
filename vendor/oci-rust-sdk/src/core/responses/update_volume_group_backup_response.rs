use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVolumeGroupBackupResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// The returned model.VolumeGroupBackup instance.
    pub volume_group_backup: VolumeGroupBackup,
}

/// Required fields for UpdateVolumeGroupBackupResponse
pub struct UpdateVolumeGroupBackupResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// The returned model.VolumeGroupBackup instance.
    pub volume_group_backup: VolumeGroupBackup,
}

impl UpdateVolumeGroupBackupResponse {
    /// Create a new UpdateVolumeGroupBackupResponse with required fields
    pub fn new(required: UpdateVolumeGroupBackupResponseRequired) -> Self {
        Self {
            etag: required.etag,

            volume_group_backup: required.volume_group_backup,
        }
    }

    /// Set etag
    pub fn set_etag(mut self, value: String) -> Self {
        self.etag = value;
        self
    }

    /// Set volume_group_backup
    pub fn set_volume_group_backup(mut self, value: VolumeGroupBackup) -> Self {
        self.volume_group_backup = value;
        self
    }
}
