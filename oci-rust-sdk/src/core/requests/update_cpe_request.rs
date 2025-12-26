use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCpeRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the CPE.
    pub cpe_id: String,

    /// Details object for updating a CPE.
    pub update_cpe_details: UpdateCpeDetails,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}


/// Required fields for UpdateCpeRequest
pub struct UpdateCpeRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the CPE.
    pub cpe_id: String,

    /// Details object for updating a CPE.
    pub update_cpe_details: UpdateCpeDetails,
}

impl UpdateCpeRequest {
    /// Create a new UpdateCpeRequest with required fields
    pub fn new(required: UpdateCpeRequestRequired) -> Self {
        Self {
            cpe_id: required.cpe_id,

            update_cpe_details: required.update_cpe_details,

            if_match: None,
}
    }

    /// Set cpe_id
    pub fn set_cpe_id(mut self, value: String) -> Self {
        self.cpe_id = value;
        self
    }

    /// Set update_cpe_details
    pub fn set_update_cpe_details(mut self, value: UpdateCpeDetails) -> Self {
        self.update_cpe_details = value;
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


