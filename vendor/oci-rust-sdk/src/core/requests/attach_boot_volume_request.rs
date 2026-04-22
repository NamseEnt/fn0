use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachBootVolumeRequest {
    /// Attach boot volume request
    pub attach_boot_volume_details: AttachBootVolumeDetails,

    /// A token that uniquely identifies a request so it can be retried in case of a timeout or server error without risk of executing that same action again. Retry tokens expire after 24 hours, but can be invalidated before then due to conflicting operations (for example, if a resource has been deleted and purged from the system, then a retry of the original creation request may be rejected).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_retry_token: Option<String>,
}

/// Required fields for AttachBootVolumeRequest
pub struct AttachBootVolumeRequestRequired {
    /// Attach boot volume request
    pub attach_boot_volume_details: AttachBootVolumeDetails,
}

impl AttachBootVolumeRequest {
    /// Create a new AttachBootVolumeRequest with required fields
    pub fn new(required: AttachBootVolumeRequestRequired) -> Self {
        Self {
            attach_boot_volume_details: required.attach_boot_volume_details,

            opc_retry_token: None,
        }
    }

    /// Set attach_boot_volume_details
    pub fn set_attach_boot_volume_details(mut self, value: AttachBootVolumeDetails) -> Self {
        self.attach_boot_volume_details = value;
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
