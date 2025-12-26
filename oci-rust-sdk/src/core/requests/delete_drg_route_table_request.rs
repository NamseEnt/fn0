use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteDrgRouteTableRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG route table.
    pub drg_route_table_id: String,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}


/// Required fields for DeleteDrgRouteTableRequest
pub struct DeleteDrgRouteTableRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG route table.
    pub drg_route_table_id: String,
}

impl DeleteDrgRouteTableRequest {
    /// Create a new DeleteDrgRouteTableRequest with required fields
    pub fn new(required: DeleteDrgRouteTableRequestRequired) -> Self {
        Self {
            drg_route_table_id: required.drg_route_table_id,

            if_match: None,
}
    }

    /// Set drg_route_table_id
    pub fn set_drg_route_table_id(mut self, value: String) -> Self {
        self.drg_route_table_id = value;
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


