use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSubnetRequest {
    /// Specify the [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the subnet.
    pub subnet_id: String,

    /// Details object for updating a subnet.
    pub update_subnet_details: UpdateSubnetDetails,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}


/// Required fields for UpdateSubnetRequest
pub struct UpdateSubnetRequestRequired {
    /// Specify the [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the subnet.
    pub subnet_id: String,

    /// Details object for updating a subnet.
    pub update_subnet_details: UpdateSubnetDetails,
}

impl UpdateSubnetRequest {
    /// Create a new UpdateSubnetRequest with required fields
    pub fn new(required: UpdateSubnetRequestRequired) -> Self {
        Self {
            subnet_id: required.subnet_id,

            update_subnet_details: required.update_subnet_details,

            if_match: None,
}
    }

    /// Set subnet_id
    pub fn set_subnet_id(mut self, value: String) -> Self {
        self.subnet_id = value;
        self
    }

    /// Set update_subnet_details
    pub fn set_update_subnet_details(mut self, value: UpdateSubnetDetails) -> Self {
        self.update_subnet_details = value;
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


