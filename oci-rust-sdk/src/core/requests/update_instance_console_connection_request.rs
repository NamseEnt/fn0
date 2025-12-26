use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstanceConsoleConnectionRequest {
    /// The OCID of the instance console connection.
    pub instance_console_connection_id: String,

    /// Update instanceConsoleConnection tags
    pub update_instance_console_connection_details: UpdateInstanceConsoleConnectionDetails,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}


/// Required fields for UpdateInstanceConsoleConnectionRequest
pub struct UpdateInstanceConsoleConnectionRequestRequired {
    /// The OCID of the instance console connection.
    pub instance_console_connection_id: String,

    /// Update instanceConsoleConnection tags
    pub update_instance_console_connection_details: UpdateInstanceConsoleConnectionDetails,
}

impl UpdateInstanceConsoleConnectionRequest {
    /// Create a new UpdateInstanceConsoleConnectionRequest with required fields
    pub fn new(required: UpdateInstanceConsoleConnectionRequestRequired) -> Self {
        Self {
            instance_console_connection_id: required.instance_console_connection_id,

            update_instance_console_connection_details: required.update_instance_console_connection_details,

            opc_request_id: None,

            if_match: None,
}
    }

    /// Set instance_console_connection_id
    pub fn set_instance_console_connection_id(mut self, value: String) -> Self {
        self.instance_console_connection_id = value;
        self
    }

    /// Set update_instance_console_connection_details
    pub fn set_update_instance_console_connection_details(mut self, value: UpdateInstanceConsoleConnectionDetails) -> Self {
        self.update_instance_console_connection_details = value;
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


