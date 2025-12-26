use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSubnetRequest {
    /// Specify the [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the subnet.
    pub subnet_id: String,
}


/// Required fields for GetSubnetRequest
pub struct GetSubnetRequestRequired {
    /// Specify the [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the subnet.
    pub subnet_id: String,
}

impl GetSubnetRequest {
    /// Create a new GetSubnetRequest with required fields
    pub fn new(required: GetSubnetRequestRequired) -> Self {
        Self {
            subnet_id: required.subnet_id,
}
    }

    /// Set subnet_id
    pub fn set_subnet_id(mut self, value: String) -> Self {
        self.subnet_id = value;
        self
    }
}


