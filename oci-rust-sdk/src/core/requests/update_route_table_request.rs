use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRouteTableRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route table.
    pub rt_id: String,

    /// Details object for updating a route table.
    pub update_route_table_details: UpdateRouteTableDetails,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}

/// Required fields for UpdateRouteTableRequest
pub struct UpdateRouteTableRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route table.
    pub rt_id: String,

    /// Details object for updating a route table.
    pub update_route_table_details: UpdateRouteTableDetails,
}

impl UpdateRouteTableRequest {
    /// Create a new UpdateRouteTableRequest with required fields
    pub fn new(required: UpdateRouteTableRequestRequired) -> Self {
        Self {
            rt_id: required.rt_id,

            update_route_table_details: required.update_route_table_details,

            if_match: None,
        }
    }

    /// Set rt_id
    pub fn set_rt_id(mut self, value: String) -> Self {
        self.rt_id = value;
        self
    }

    /// Set update_route_table_details
    pub fn set_update_route_table_details(mut self, value: UpdateRouteTableDetails) -> Self {
        self.update_route_table_details = value;
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
