use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVolumeBackupPolicyResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.VolumeBackupPolicy instance.
    pub volume_backup_policy: VolumeBackupPolicy,
}


/// Required fields for GetVolumeBackupPolicyResponse
pub struct GetVolumeBackupPolicyResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.VolumeBackupPolicy instance.
    pub volume_backup_policy: VolumeBackupPolicy,
}

impl GetVolumeBackupPolicyResponse {
    /// Create a new GetVolumeBackupPolicyResponse with required fields
    pub fn new(required: GetVolumeBackupPolicyResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            volume_backup_policy: required.volume_backup_policy,
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

    /// Set volume_backup_policy
    pub fn set_volume_backup_policy(mut self, value: VolumeBackupPolicy) -> Self {
        self.volume_backup_policy = value;
        self
    }
}


