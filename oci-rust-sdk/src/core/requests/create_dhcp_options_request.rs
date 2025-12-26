use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDhcpOptionsRequest {
    /// Request object for creating a new set of DHCP options.
    pub create_dhcp_details: CreateDhcpDetails,

    /// A token that uniquely identifies a request so it can be retried in case of a timeout or server error without risk of executing that same action again. Retry tokens expire after 24 hours, but can be invalidated before then due to conflicting operations (for example, if a resource has been deleted and purged from the system, then a retry of the original creation request may be rejected).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_retry_token: Option<String>,
}


/// Required fields for CreateDhcpOptionsRequest
pub struct CreateDhcpOptionsRequestRequired {
    /// Request object for creating a new set of DHCP options.
    pub create_dhcp_details: CreateDhcpDetails,
}

impl CreateDhcpOptionsRequest {
    /// Create a new CreateDhcpOptionsRequest with required fields
    pub fn new(required: CreateDhcpOptionsRequestRequired) -> Self {
        Self {
            create_dhcp_details: required.create_dhcp_details,

            opc_retry_token: None,
}
    }

    /// Set create_dhcp_details
    pub fn set_create_dhcp_details(mut self, value: CreateDhcpDetails) -> Self {
        self.create_dhcp_details = value;
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


