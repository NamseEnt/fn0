use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetNatGatewayRequest {
    /// The NAT gateway's [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm).
    pub nat_gateway_id: String,
}


/// Required fields for GetNatGatewayRequest
pub struct GetNatGatewayRequestRequired {
    /// The NAT gateway's [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm).
    pub nat_gateway_id: String,
}

impl GetNatGatewayRequest {
    /// Create a new GetNatGatewayRequest with required fields
    pub fn new(required: GetNatGatewayRequestRequired) -> Self {
        Self {
            nat_gateway_id: required.nat_gateway_id,
}
    }

    /// Set nat_gateway_id
    pub fn set_nat_gateway_id(mut self, value: String) -> Self {
        self.nat_gateway_id = value;
        self
    }
}


