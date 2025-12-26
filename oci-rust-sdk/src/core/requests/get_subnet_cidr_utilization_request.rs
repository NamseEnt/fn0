use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSubnetCidrUtilizationRequest {
    /// Specify the [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the subnet.
    pub subnet_id: String,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}


/// Required fields for GetSubnetCidrUtilizationRequest
pub struct GetSubnetCidrUtilizationRequestRequired {
    /// Specify the [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the subnet.
    pub subnet_id: String,
}

impl GetSubnetCidrUtilizationRequest {
    /// Create a new GetSubnetCidrUtilizationRequest with required fields
    pub fn new(required: GetSubnetCidrUtilizationRequestRequired) -> Self {
        Self {
            subnet_id: required.subnet_id,

            opc_request_id: None,
}
    }

    /// Set subnet_id
    pub fn set_subnet_id(mut self, value: String) -> Self {
        self.subnet_id = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set opc_request_id (unwraps Option)
    pub fn with_opc_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_request_id = Some(value.into());
        self
    }
}


