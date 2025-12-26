use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVolumeBackupResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.VolumeBackup instance.
    pub volume_backup: VolumeBackup,
}


/// Required fields for CreateVolumeBackupResponse
pub struct CreateVolumeBackupResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.VolumeBackup instance.
    pub volume_backup: VolumeBackup,
}

impl CreateVolumeBackupResponse {
    /// Create a new CreateVolumeBackupResponse with required fields
    pub fn new(required: CreateVolumeBackupResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            volume_backup: required.volume_backup,
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

    /// Set volume_backup
    pub fn set_volume_backup(mut self, value: VolumeBackup) -> Self {
        self.volume_backup = value;
        self
    }
}


