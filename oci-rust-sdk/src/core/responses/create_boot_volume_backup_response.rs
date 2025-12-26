use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBootVolumeBackupResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.BootVolumeBackup instance.
    pub boot_volume_backup: BootVolumeBackup,
}


/// Required fields for CreateBootVolumeBackupResponse
pub struct CreateBootVolumeBackupResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.BootVolumeBackup instance.
    pub boot_volume_backup: BootVolumeBackup,
}

impl CreateBootVolumeBackupResponse {
    /// Create a new CreateBootVolumeBackupResponse with required fields
    pub fn new(required: CreateBootVolumeBackupResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            boot_volume_backup: required.boot_volume_backup,
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

    /// Set boot_volume_backup
    pub fn set_boot_volume_backup(mut self, value: BootVolumeBackup) -> Self {
        self.boot_volume_backup = value;
        self
    }
}


