use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInternetGatewayRequest {
    /// Details for creating a new internet gateway.
    pub create_internet_gateway_details: CreateInternetGatewayDetails,

    /// A token that uniquely identifies a request so it can be retried in case of a timeout or server error without risk of executing that same action again. Retry tokens expire after 24 hours, but can be invalidated before then due to conflicting operations (for example, if a resource has been deleted and purged from the system, then a retry of the original creation request may be rejected).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_retry_token: Option<String>,
}


/// Required fields for CreateInternetGatewayRequest
pub struct CreateInternetGatewayRequestRequired {
    /// Details for creating a new internet gateway.
    pub create_internet_gateway_details: CreateInternetGatewayDetails,
}

impl CreateInternetGatewayRequest {
    /// Create a new CreateInternetGatewayRequest with required fields
    pub fn new(required: CreateInternetGatewayRequestRequired) -> Self {
        Self {
            create_internet_gateway_details: required.create_internet_gateway_details,

            opc_retry_token: None,
}
    }

    /// Set create_internet_gateway_details
    pub fn set_create_internet_gateway_details(mut self, value: CreateInternetGatewayDetails) -> Self {
        self.create_internet_gateway_details = value;
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


