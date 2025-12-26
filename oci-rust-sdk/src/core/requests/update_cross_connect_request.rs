use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCrossConnectRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the cross-connect.
    pub cross_connect_id: String,

    /// Update CrossConnect fields.
    pub update_cross_connect_details: UpdateCrossConnectDetails,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}


/// Required fields for UpdateCrossConnectRequest
pub struct UpdateCrossConnectRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the cross-connect.
    pub cross_connect_id: String,

    /// Update CrossConnect fields.
    pub update_cross_connect_details: UpdateCrossConnectDetails,
}

impl UpdateCrossConnectRequest {
    /// Create a new UpdateCrossConnectRequest with required fields
    pub fn new(required: UpdateCrossConnectRequestRequired) -> Self {
        Self {
            cross_connect_id: required.cross_connect_id,

            update_cross_connect_details: required.update_cross_connect_details,

            if_match: None,
}
    }

    /// Set cross_connect_id
    pub fn set_cross_connect_id(mut self, value: String) -> Self {
        self.cross_connect_id = value;
        self
    }

    /// Set update_cross_connect_details
    pub fn set_update_cross_connect_details(mut self, value: UpdateCrossConnectDetails) -> Self {
        self.update_cross_connect_details = value;
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


