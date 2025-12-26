use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateContainerRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the container.
    pub container_id: String,

    /// The information to be updated.
    pub update_container_details: UpdateContainerDetails,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}


/// Required fields for UpdateContainerRequest
pub struct UpdateContainerRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the container.
    pub container_id: String,

    /// The information to be updated.
    pub update_container_details: UpdateContainerDetails,
}

impl UpdateContainerRequest {
    /// Create a new UpdateContainerRequest with required fields
    pub fn new(required: UpdateContainerRequestRequired) -> Self {
        Self {
            container_id: required.container_id,

            update_container_details: required.update_container_details,

            if_match: None,

            opc_request_id: None,
}
    }

    /// Set container_id
    pub fn set_container_id(mut self, value: String) -> Self {
        self.container_id = value;
        self
    }

    /// Set update_container_details
    pub fn set_update_container_details(mut self, value: UpdateContainerDetails) -> Self {
        self.update_container_details = value;
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
    /// Convert this request's query parameters to a vector of key-value pairs.
    pub fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();

        if let Some(ref if_match) = self.if_match {
            params.push(("if_match".to_string(), if_match.clone()));
        }

        params
    }

}


