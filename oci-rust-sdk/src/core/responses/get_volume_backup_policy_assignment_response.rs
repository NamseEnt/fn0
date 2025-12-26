use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVolumeBackupPolicyAssignmentResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.VolumeBackupPolicyAssignment instance.
    pub volume_backup_policy_assignment: VolumeBackupPolicyAssignment,
}

/// Required fields for GetVolumeBackupPolicyAssignmentResponse
pub struct GetVolumeBackupPolicyAssignmentResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.VolumeBackupPolicyAssignment instance.
    pub volume_backup_policy_assignment: VolumeBackupPolicyAssignment,
}

impl GetVolumeBackupPolicyAssignmentResponse {
    /// Create a new GetVolumeBackupPolicyAssignmentResponse with required fields
    pub fn new(required: GetVolumeBackupPolicyAssignmentResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            volume_backup_policy_assignment: required.volume_backup_policy_assignment,
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

    /// Set volume_backup_policy_assignment
    pub fn set_volume_backup_policy_assignment(
        mut self,
        value: VolumeBackupPolicyAssignment,
    ) -> Self {
        self.volume_backup_policy_assignment = value;
        self
    }
}
