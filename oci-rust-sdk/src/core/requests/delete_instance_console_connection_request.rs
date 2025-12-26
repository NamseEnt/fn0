use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteInstanceConsoleConnectionRequest {
    /// The OCID of the instance console connection.
    pub instance_console_connection_id: String,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}

/// Required fields for DeleteInstanceConsoleConnectionRequest
pub struct DeleteInstanceConsoleConnectionRequestRequired {
    /// The OCID of the instance console connection.
    pub instance_console_connection_id: String,
}

impl DeleteInstanceConsoleConnectionRequest {
    /// Create a new DeleteInstanceConsoleConnectionRequest with required fields
    pub fn new(required: DeleteInstanceConsoleConnectionRequestRequired) -> Self {
        Self {
            instance_console_connection_id: required.instance_console_connection_id,

            if_match: None,
        }
    }

    /// Set instance_console_connection_id
    pub fn set_instance_console_connection_id(mut self, value: String) -> Self {
        self.instance_console_connection_id = value;
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
