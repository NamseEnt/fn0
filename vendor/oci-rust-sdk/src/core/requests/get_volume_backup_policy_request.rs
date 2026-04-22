use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVolumeBackupPolicyRequest {
    /// The OCID of the volume backup policy.
    pub policy_id: String,
}

/// Required fields for GetVolumeBackupPolicyRequest
pub struct GetVolumeBackupPolicyRequestRequired {
    /// The OCID of the volume backup policy.
    pub policy_id: String,
}

impl GetVolumeBackupPolicyRequest {
    /// Create a new GetVolumeBackupPolicyRequest with required fields
    pub fn new(required: GetVolumeBackupPolicyRequestRequired) -> Self {
        Self {
            policy_id: required.policy_id,
        }
    }

    /// Set policy_id
    pub fn set_policy_id(mut self, value: String) -> Self {
        self.policy_id = value;
        self
    }
}
