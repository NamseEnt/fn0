use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetInstanceConsoleConnectionResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.InstanceConsoleConnection instance.
    pub instance_console_connection: InstanceConsoleConnection,
}


/// Required fields for GetInstanceConsoleConnectionResponse
pub struct GetInstanceConsoleConnectionResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.InstanceConsoleConnection instance.
    pub instance_console_connection: InstanceConsoleConnection,
}

impl GetInstanceConsoleConnectionResponse {
    /// Create a new GetInstanceConsoleConnectionResponse with required fields
    pub fn new(required: GetInstanceConsoleConnectionResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,

            instance_console_connection: required.instance_console_connection,
}
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set instance_console_connection
    pub fn set_instance_console_connection(mut self, value: InstanceConsoleConnection) -> Self {
        self.instance_console_connection = value;
        self
    }
}


