use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDrgRouteTableRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG route table.
    pub drg_route_table_id: String,

    /// Details object used to updating a DRG route table.
    pub update_drg_route_table_details: UpdateDrgRouteTableDetails,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}

/// Required fields for UpdateDrgRouteTableRequest
pub struct UpdateDrgRouteTableRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG route table.
    pub drg_route_table_id: String,

    /// Details object used to updating a DRG route table.
    pub update_drg_route_table_details: UpdateDrgRouteTableDetails,
}

impl UpdateDrgRouteTableRequest {
    /// Create a new UpdateDrgRouteTableRequest with required fields
    pub fn new(required: UpdateDrgRouteTableRequestRequired) -> Self {
        Self {
            drg_route_table_id: required.drg_route_table_id,

            update_drg_route_table_details: required.update_drg_route_table_details,

            if_match: None,
        }
    }

    /// Set drg_route_table_id
    pub fn set_drg_route_table_id(mut self, value: String) -> Self {
        self.drg_route_table_id = value;
        self
    }

    /// Set update_drg_route_table_details
    pub fn set_update_drg_route_table_details(mut self, value: UpdateDrgRouteTableDetails) -> Self {
        self.update_drg_route_table_details = value;
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
