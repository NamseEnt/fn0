use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePrivateIpRequest {
    /// Create private IP details.
    pub create_private_ip_details: CreatePrivateIpDetails,

    /// A token that uniquely identifies a request so it can be retried in case of a timeout or server error without risk of executing that same action again. Retry tokens expire after 24 hours, but can be invalidated before then due to conflicting operations (for example, if a resource has been deleted and purged from the system, then a retry of the original creation request may be rejected).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_retry_token: Option<String>,
}

/// Required fields for CreatePrivateIpRequest
pub struct CreatePrivateIpRequestRequired {
    /// Create private IP details.
    pub create_private_ip_details: CreatePrivateIpDetails,
}

impl CreatePrivateIpRequest {
    /// Create a new CreatePrivateIpRequest with required fields
    pub fn new(required: CreatePrivateIpRequestRequired) -> Self {
        Self {
            create_private_ip_details: required.create_private_ip_details,

            opc_retry_token: None,
        }
    }

    /// Set create_private_ip_details
    pub fn set_create_private_ip_details(mut self, value: CreatePrivateIpDetails) -> Self {
        self.create_private_ip_details = value;
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
