use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateByoasnRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the {@code Byoasn} resource.
    pub byoasn_id: String,

    /// Byoasn Range details.
    pub update_byoasn_details: UpdateByoasnDetails,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}

/// Required fields for UpdateByoasnRequest
pub struct UpdateByoasnRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the {@code Byoasn} resource.
    pub byoasn_id: String,

    /// Byoasn Range details.
    pub update_byoasn_details: UpdateByoasnDetails,
}

impl UpdateByoasnRequest {
    /// Create a new UpdateByoasnRequest with required fields
    pub fn new(required: UpdateByoasnRequestRequired) -> Self {
        Self {
            byoasn_id: required.byoasn_id,

            update_byoasn_details: required.update_byoasn_details,

            opc_request_id: None,

            if_match: None,
        }
    }

    /// Set byoasn_id
    pub fn set_byoasn_id(mut self, value: String) -> Self {
        self.byoasn_id = value;
        self
    }

    /// Set update_byoasn_details
    pub fn set_update_byoasn_details(mut self, value: UpdateByoasnDetails) -> Self {
        self.update_byoasn_details = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set if_match
    pub fn set_if_match(mut self, value: Option<String>) -> Self {
        self.if_match = value;
        self
    }

    /// Set opc_request_id (unwraps Option)
    pub fn with_opc_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_request_id = Some(value.into());
        self
    }

    /// Set if_match (unwraps Option)
    pub fn with_if_match(mut self, value: impl Into<String>) -> Self {
        self.if_match = Some(value.into());
        self
    }
}
