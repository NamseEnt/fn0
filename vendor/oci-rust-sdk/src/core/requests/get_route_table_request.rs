use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRouteTableRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route table.
    pub rt_id: String,
}

/// Required fields for GetRouteTableRequest
pub struct GetRouteTableRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route table.
    pub rt_id: String,
}

impl GetRouteTableRequest {
    /// Create a new GetRouteTableRequest with required fields
    pub fn new(required: GetRouteTableRequestRequired) -> Self {
        Self {
            rt_id: required.rt_id,
        }
    }

    /// Set rt_id
    pub fn set_rt_id(mut self, value: String) -> Self {
        self.rt_id = value;
        self
    }
}
