use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVolumeBackupPolicyRequest {
    /// Request to create a new scheduled backup policy.
    pub create_volume_backup_policy_details: CreateVolumeBackupPolicyDetails,

    /// A token that uniquely identifies a request so it can be retried in case of a timeout or server error without risk of executing that same action again. Retry tokens expire after 24 hours, but can be invalidated before then due to conflicting operations (for example, if a resource has been deleted and purged from the system, then a retry of the original creation request may be rejected).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_retry_token: Option<String>,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}


/// Required fields for CreateVolumeBackupPolicyRequest
pub struct CreateVolumeBackupPolicyRequestRequired {
    /// Request to create a new scheduled backup policy.
    pub create_volume_backup_policy_details: CreateVolumeBackupPolicyDetails,
}

impl CreateVolumeBackupPolicyRequest {
    /// Create a new CreateVolumeBackupPolicyRequest with required fields
    pub fn new(required: CreateVolumeBackupPolicyRequestRequired) -> Self {
        Self {
            create_volume_backup_policy_details: required.create_volume_backup_policy_details,

            opc_retry_token: None,

            opc_request_id: None,
}
    }

    /// Set create_volume_backup_policy_details
    pub fn set_create_volume_backup_policy_details(mut self, value: CreateVolumeBackupPolicyDetails) -> Self {
        self.create_volume_backup_policy_details = value;
        self
    }

    /// Set opc_retry_token
    pub fn set_opc_retry_token(mut self, value: Option<String>) -> Self {
        self.opc_retry_token = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set opc_retry_token (unwraps Option)
    pub fn with_opc_retry_token(mut self, value: impl Into<String>) -> Self {
        self.opc_retry_token = Some(value.into());
        self
    }

    /// Set opc_request_id (unwraps Option)
    pub fn with_opc_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_request_id = Some(value.into());
        self
    }
}


