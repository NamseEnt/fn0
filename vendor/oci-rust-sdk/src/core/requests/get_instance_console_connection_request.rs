use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetInstanceConsoleConnectionRequest {
    /// The OCID of the instance console connection.
    pub instance_console_connection_id: String,
}

/// Required fields for GetInstanceConsoleConnectionRequest
pub struct GetInstanceConsoleConnectionRequestRequired {
    /// The OCID of the instance console connection.
    pub instance_console_connection_id: String,
}

impl GetInstanceConsoleConnectionRequest {
    /// Create a new GetInstanceConsoleConnectionRequest with required fields
    pub fn new(required: GetInstanceConsoleConnectionRequestRequired) -> Self {
        Self {
            instance_console_connection_id: required.instance_console_connection_id,
        }
    }

    /// Set instance_console_connection_id
    pub fn set_instance_console_connection_id(mut self, value: String) -> Self {
        self.instance_console_connection_id = value;
        self
    }
}
