use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVolumeBackupPolicyAssignmentRequest {
    /// Request to assign a specified policy to a particular volume.
    pub create_volume_backup_policy_assignment_details: CreateVolumeBackupPolicyAssignmentDetails,
}


/// Required fields for CreateVolumeBackupPolicyAssignmentRequest
pub struct CreateVolumeBackupPolicyAssignmentRequestRequired {
    /// Request to assign a specified policy to a particular volume.
    pub create_volume_backup_policy_assignment_details: CreateVolumeBackupPolicyAssignmentDetails,
}

impl CreateVolumeBackupPolicyAssignmentRequest {
    /// Create a new CreateVolumeBackupPolicyAssignmentRequest with required fields
    pub fn new(required: CreateVolumeBackupPolicyAssignmentRequestRequired) -> Self {
        Self {
            create_volume_backup_policy_assignment_details: required.create_volume_backup_policy_assignment_details,
}
    }

    /// Set create_volume_backup_policy_assignment_details
    pub fn set_create_volume_backup_policy_assignment_details(mut self, value: CreateVolumeBackupPolicyAssignmentDetails) -> Self {
        self.create_volume_backup_policy_assignment_details = value;
        self
    }
}


