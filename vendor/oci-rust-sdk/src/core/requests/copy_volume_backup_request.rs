use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyVolumeBackupRequest {
    /// The OCID of the volume backup.
    pub volume_backup_id: String,

    /// Request to create a cross-region copy of given backup.
    pub copy_volume_backup_details: CopyVolumeBackupDetails,

    /// A token that uniquely identifies a request so it can be retried in case of a timeout or server error without risk of executing that same action again. Retry tokens expire after 24 hours, but can be invalidated before then due to conflicting operations (for example, if a resource has been deleted and purged from the system, then a retry of the original creation request may be rejected).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_retry_token: Option<String>,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}

/// Required fields for CopyVolumeBackupRequest
pub struct CopyVolumeBackupRequestRequired {
    /// The OCID of the volume backup.
    pub volume_backup_id: String,

    /// Request to create a cross-region copy of given backup.
    pub copy_volume_backup_details: CopyVolumeBackupDetails,
}

impl CopyVolumeBackupRequest {
    /// Create a new CopyVolumeBackupRequest with required fields
    pub fn new(required: CopyVolumeBackupRequestRequired) -> Self {
        Self {
            volume_backup_id: required.volume_backup_id,

            copy_volume_backup_details: required.copy_volume_backup_details,

            opc_retry_token: None,

            opc_request_id: None,
        }
    }

    /// Set volume_backup_id
    pub fn set_volume_backup_id(mut self, value: String) -> Self {
        self.volume_backup_id = value;
        self
    }

    /// Set copy_volume_backup_details
    pub fn set_copy_volume_backup_details(mut self, value: CopyVolumeBackupDetails) -> Self {
        self.copy_volume_backup_details = value;
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
