use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVolumeGroupBackupResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.VolumeGroupBackup instance.
    pub volume_group_backup: VolumeGroupBackup,
}


/// Required fields for CreateVolumeGroupBackupResponse
pub struct CreateVolumeGroupBackupResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.VolumeGroupBackup instance.
    pub volume_group_backup: VolumeGroupBackup,
}

impl CreateVolumeGroupBackupResponse {
    /// Create a new CreateVolumeGroupBackupResponse with required fields
    pub fn new(required: CreateVolumeGroupBackupResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            volume_group_backup: required.volume_group_backup,
}
    }

    /// Set etag
    pub fn set_etag(mut self, value: String) -> Self {
        self.etag = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set volume_group_backup
    pub fn set_volume_group_backup(mut self, value: VolumeGroupBackup) -> Self {
        self.volume_group_backup = value;
        self
    }
}


