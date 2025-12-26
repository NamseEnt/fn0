use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVnicRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VNIC.
    pub vnic_id: String,

    /// Details object for updating a VNIC.
    pub update_vnic_details: UpdateVnicDetails,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}


/// Required fields for UpdateVnicRequest
pub struct UpdateVnicRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VNIC.
    pub vnic_id: String,

    /// Details object for updating a VNIC.
    pub update_vnic_details: UpdateVnicDetails,
}

impl UpdateVnicRequest {
    /// Create a new UpdateVnicRequest with required fields
    pub fn new(required: UpdateVnicRequestRequired) -> Self {
        Self {
            vnic_id: required.vnic_id,

            update_vnic_details: required.update_vnic_details,

            if_match: None,
}
    }

    /// Set vnic_id
    pub fn set_vnic_id(mut self, value: String) -> Self {
        self.vnic_id = value;
        self
    }

    /// Set update_vnic_details
    pub fn set_update_vnic_details(mut self, value: UpdateVnicDetails) -> Self {
        self.update_vnic_details = value;
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


