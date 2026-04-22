use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVcnRequest {
    /// Specify the [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN.
    pub vcn_id: String,

    /// Details object for updating a VCN.
    pub update_vcn_details: UpdateVcnDetails,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}

/// Required fields for UpdateVcnRequest
pub struct UpdateVcnRequestRequired {
    /// Specify the [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN.
    pub vcn_id: String,

    /// Details object for updating a VCN.
    pub update_vcn_details: UpdateVcnDetails,
}

impl UpdateVcnRequest {
    /// Create a new UpdateVcnRequest with required fields
    pub fn new(required: UpdateVcnRequestRequired) -> Self {
        Self {
            vcn_id: required.vcn_id,

            update_vcn_details: required.update_vcn_details,

            if_match: None,
        }
    }

    /// Set vcn_id
    pub fn set_vcn_id(mut self, value: String) -> Self {
        self.vcn_id = value;
        self
    }

    /// Set update_vcn_details
    pub fn set_update_vcn_details(mut self, value: UpdateVcnDetails) -> Self {
        self.update_vcn_details = value;
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
