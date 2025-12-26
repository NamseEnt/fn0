use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteLocalPeeringGatewayRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the local peering gateway.
    pub local_peering_gateway_id: String,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}


/// Required fields for DeleteLocalPeeringGatewayRequest
pub struct DeleteLocalPeeringGatewayRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the local peering gateway.
    pub local_peering_gateway_id: String,
}

impl DeleteLocalPeeringGatewayRequest {
    /// Create a new DeleteLocalPeeringGatewayRequest with required fields
    pub fn new(required: DeleteLocalPeeringGatewayRequestRequired) -> Self {
        Self {
            local_peering_gateway_id: required.local_peering_gateway_id,

            if_match: None,
}
    }

    /// Set local_peering_gateway_id
    pub fn set_local_peering_gateway_id(mut self, value: String) -> Self {
        self.local_peering_gateway_id = value;
        self
    }

    /// Set if_match
    pub fn set_if_match(mut self, value: Option<String>) -> Self {
        self.if_match = value;
        self
    }

    /// Set if_match (unwraps Option)
    pub fn with_if_match(mut self, value: impl Into<String>) -> Self {
        self.if_match = Some(value.into());
        self
    }
}


