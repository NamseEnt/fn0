use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateByoipRangeRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the {@code ByoipRange} resource containing the BYOIP CIDR block.
    pub byoip_range_id: String,

    /// Byoip Range details.
    pub update_byoip_range_details: UpdateByoipRangeDetails,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}


/// Required fields for UpdateByoipRangeRequest
pub struct UpdateByoipRangeRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the {@code ByoipRange} resource containing the BYOIP CIDR block.
    pub byoip_range_id: String,

    /// Byoip Range details.
    pub update_byoip_range_details: UpdateByoipRangeDetails,
}

impl UpdateByoipRangeRequest {
    /// Create a new UpdateByoipRangeRequest with required fields
    pub fn new(required: UpdateByoipRangeRequestRequired) -> Self {
        Self {
            byoip_range_id: required.byoip_range_id,

            update_byoip_range_details: required.update_byoip_range_details,

            opc_request_id: None,

            if_match: None,
}
    }

    /// Set byoip_range_id
    pub fn set_byoip_range_id(mut self, value: String) -> Self {
        self.byoip_range_id = value;
        self
    }

    /// Set update_byoip_range_details
    pub fn set_update_byoip_range_details(mut self, value: UpdateByoipRangeDetails) -> Self {
        self.update_byoip_range_details = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set if_match
    pub fn set_if_match(mut self, value: Option<String>) -> Self {
        self.if_match = value;
        self
    }

    /// Set opc_request_id (unwraps Option)
    pub fn with_opc_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_request_id = Some(value.into());
        self
    }

    /// Set if_match (unwraps Option)
    pub fn with_if_match(mut self, value: impl Into<String>) -> Self {
        self.if_match = Some(value.into());
        self
    }
}


