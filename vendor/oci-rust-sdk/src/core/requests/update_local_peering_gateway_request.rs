use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLocalPeeringGatewayRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the local peering gateway.
    pub local_peering_gateway_id: String,

    /// Details object for updating a local peering gateway.
    pub update_local_peering_gateway_details: UpdateLocalPeeringGatewayDetails,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}

/// Required fields for UpdateLocalPeeringGatewayRequest
pub struct UpdateLocalPeeringGatewayRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the local peering gateway.
    pub local_peering_gateway_id: String,

    /// Details object for updating a local peering gateway.
    pub update_local_peering_gateway_details: UpdateLocalPeeringGatewayDetails,
}

impl UpdateLocalPeeringGatewayRequest {
    /// Create a new UpdateLocalPeeringGatewayRequest with required fields
    pub fn new(required: UpdateLocalPeeringGatewayRequestRequired) -> Self {
        Self {
            local_peering_gateway_id: required.local_peering_gateway_id,

            update_local_peering_gateway_details: required.update_local_peering_gateway_details,

            if_match: None,
        }
    }

    /// Set local_peering_gateway_id
    pub fn set_local_peering_gateway_id(mut self, value: String) -> Self {
        self.local_peering_gateway_id = value;
        self
    }

    /// Set update_local_peering_gateway_details
    pub fn set_update_local_peering_gateway_details(
        mut self,
        value: UpdateLocalPeeringGatewayDetails,
    ) -> Self {
        self.update_local_peering_gateway_details = value;
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
