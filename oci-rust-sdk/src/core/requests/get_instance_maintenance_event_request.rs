use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetInstanceMaintenanceEventRequest {
    /// The OCID of the instance maintenance event.
    pub instance_maintenance_event_id: String,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}


/// Required fields for GetInstanceMaintenanceEventRequest
pub struct GetInstanceMaintenanceEventRequestRequired {
    /// The OCID of the instance maintenance event.
    pub instance_maintenance_event_id: String,
}

impl GetInstanceMaintenanceEventRequest {
    /// Create a new GetInstanceMaintenanceEventRequest with required fields
    pub fn new(required: GetInstanceMaintenanceEventRequestRequired) -> Self {
        Self {
            instance_maintenance_event_id: required.instance_maintenance_event_id,

            opc_request_id: None,
}
    }

    /// Set instance_maintenance_event_id
    pub fn set_instance_maintenance_event_id(mut self, value: String) -> Self {
        self.instance_maintenance_event_id = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set opc_request_id (unwraps Option)
    pub fn with_opc_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_request_id = Some(value.into());
        self
    }
}


