use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetServiceGatewayRequest {
    /// The service gateway's [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm).
    pub service_gateway_id: String,
}

/// Required fields for GetServiceGatewayRequest
pub struct GetServiceGatewayRequestRequired {
    /// The service gateway's [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm).
    pub service_gateway_id: String,
}

impl GetServiceGatewayRequest {
    /// Create a new GetServiceGatewayRequest with required fields
    pub fn new(required: GetServiceGatewayRequestRequired) -> Self {
        Self {
            service_gateway_id: required.service_gateway_id,
        }
    }

    /// Set service_gateway_id
    pub fn set_service_gateway_id(mut self, value: String) -> Self {
        self.service_gateway_id = value;
        self
    }
}
