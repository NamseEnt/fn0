use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVtapRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VTAP.
    pub vtap_id: String,

    /// Details object for updating a VTAP.
    pub update_vtap_details: UpdateVtapDetails,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}


/// Required fields for UpdateVtapRequest
pub struct UpdateVtapRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VTAP.
    pub vtap_id: String,

    /// Details object for updating a VTAP.
    pub update_vtap_details: UpdateVtapDetails,
}

impl UpdateVtapRequest {
    /// Create a new UpdateVtapRequest with required fields
    pub fn new(required: UpdateVtapRequestRequired) -> Self {
        Self {
            vtap_id: required.vtap_id,

            update_vtap_details: required.update_vtap_details,

            if_match: None,

            opc_request_id: None,
}
    }

    /// Set vtap_id
    pub fn set_vtap_id(mut self, value: String) -> Self {
        self.vtap_id = value;
        self
    }

    /// Set update_vtap_details
    pub fn set_update_vtap_details(mut self, value: UpdateVtapDetails) -> Self {
        self.update_vtap_details = value;
        self
    }

    /// Set if_match
    pub fn set_if_match(mut self, value: Option<String>) -> Self {
        self.if_match = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set if_match (unwraps Option)
    pub fn with_if_match(mut self, value: impl Into<String>) -> Self {
        self.if_match = Some(value.into());
        self
    }

    /// Set opc_request_id (unwraps Option)
    pub fn with_opc_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_request_id = Some(value.into());
        self
    }
}


