use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBootVolumeBackupRequest {
    /// Request to create a new backup of given boot volume.
    pub create_boot_volume_backup_details: CreateBootVolumeBackupDetails,

    /// A token that uniquely identifies a request so it can be retried in case of a timeout or server error without risk of executing that same action again. Retry tokens expire after 24 hours, but can be invalidated before then due to conflicting operations (for example, if a resource has been deleted and purged from the system, then a retry of the original creation request may be rejected).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_retry_token: Option<String>,
}

/// Required fields for CreateBootVolumeBackupRequest
pub struct CreateBootVolumeBackupRequestRequired {
    /// Request to create a new backup of given boot volume.
    pub create_boot_volume_backup_details: CreateBootVolumeBackupDetails,
}

impl CreateBootVolumeBackupRequest {
    /// Create a new CreateBootVolumeBackupRequest with required fields
    pub fn new(required: CreateBootVolumeBackupRequestRequired) -> Self {
        Self {
            create_boot_volume_backup_details: required.create_boot_volume_backup_details,

            opc_retry_token: None,
        }
    }

    /// Set create_boot_volume_backup_details
    pub fn set_create_boot_volume_backup_details(
        mut self,
        value: CreateBootVolumeBackupDetails,
    ) -> Self {
        self.create_boot_volume_backup_details = value;
        self
    }

    /// Set opc_retry_token
    pub fn set_opc_retry_token(mut self, value: Option<String>) -> Self {
        self.opc_retry_token = value;
        self
    }

    /// Set opc_retry_token (unwraps Option)
    pub fn with_opc_retry_token(mut self, value: impl Into<String>) -> Self {
        self.opc_retry_token = Some(value.into());
        self
    }
}
