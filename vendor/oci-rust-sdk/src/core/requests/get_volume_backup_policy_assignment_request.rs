use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVolumeBackupPolicyAssignmentRequest {
    /// The OCID of the volume backup policy assignment.
    pub policy_assignment_id: String,
}

/// Required fields for GetVolumeBackupPolicyAssignmentRequest
pub struct GetVolumeBackupPolicyAssignmentRequestRequired {
    /// The OCID of the volume backup policy assignment.
    pub policy_assignment_id: String,
}

impl GetVolumeBackupPolicyAssignmentRequest {
    /// Create a new GetVolumeBackupPolicyAssignmentRequest with required fields
    pub fn new(required: GetVolumeBackupPolicyAssignmentRequestRequired) -> Self {
        Self {
            policy_assignment_id: required.policy_assignment_id,
        }
    }

    /// Set policy_assignment_id
    pub fn set_policy_assignment_id(mut self, value: String) -> Self {
        self.policy_assignment_id = value;
        self
    }
}
