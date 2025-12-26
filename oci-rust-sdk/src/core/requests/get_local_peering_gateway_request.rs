use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetLocalPeeringGatewayRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the local peering gateway.
    pub local_peering_gateway_id: String,
}

/// Required fields for GetLocalPeeringGatewayRequest
pub struct GetLocalPeeringGatewayRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the local peering gateway.
    pub local_peering_gateway_id: String,
}

impl GetLocalPeeringGatewayRequest {
    /// Create a new GetLocalPeeringGatewayRequest with required fields
    pub fn new(required: GetLocalPeeringGatewayRequestRequired) -> Self {
        Self {
            local_peering_gateway_id: required.local_peering_gateway_id,
        }
    }

    /// Set local_peering_gateway_id
    pub fn set_local_peering_gateway_id(mut self, value: String) -> Self {
        self.local_peering_gateway_id = value;
        self
    }
}
