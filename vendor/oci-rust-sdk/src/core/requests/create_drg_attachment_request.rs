use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDrgAttachmentRequest {
    /// Details for creating a {@code DrgAttachment}.
    pub create_drg_attachment_details: CreateDrgAttachmentDetails,

    /// A token that uniquely identifies a request so it can be retried in case of a timeout or server error without risk of executing that same action again. Retry tokens expire after 24 hours, but can be invalidated before then due to conflicting operations (for example, if a resource has been deleted and purged from the system, then a retry of the original creation request may be rejected).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_retry_token: Option<String>,
}

/// Required fields for CreateDrgAttachmentRequest
pub struct CreateDrgAttachmentRequestRequired {
    /// Details for creating a {@code DrgAttachment}.
    pub create_drg_attachment_details: CreateDrgAttachmentDetails,
}

impl CreateDrgAttachmentRequest {
    /// Create a new CreateDrgAttachmentRequest with required fields
    pub fn new(required: CreateDrgAttachmentRequestRequired) -> Self {
        Self {
            create_drg_attachment_details: required.create_drg_attachment_details,

            opc_retry_token: None,
        }
    }

    /// Set create_drg_attachment_details
    pub fn set_create_drg_attachment_details(mut self, value: CreateDrgAttachmentDetails) -> Self {
        self.create_drg_attachment_details = value;
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
