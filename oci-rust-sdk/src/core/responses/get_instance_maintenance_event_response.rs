use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetInstanceMaintenanceEventResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.InstanceMaintenanceEvent instance.
    pub instance_maintenance_event: InstanceMaintenanceEvent,
}


/// Required fields for GetInstanceMaintenanceEventResponse
pub struct GetInstanceMaintenanceEventResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.InstanceMaintenanceEvent instance.
    pub instance_maintenance_event: InstanceMaintenanceEvent,
}

impl GetInstanceMaintenanceEventResponse {
    /// Create a new GetInstanceMaintenanceEventResponse with required fields
    pub fn new(required: GetInstanceMaintenanceEventResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            instance_maintenance_event: required.instance_maintenance_event,
}
    }

    /// Set etag
    pub fn set_etag(mut self, value: String) -> Self {
        self.etag = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set instance_maintenance_event
    pub fn set_instance_maintenance_event(mut self, value: InstanceMaintenanceEvent) -> Self {
        self.instance_maintenance_event = value;
        self
    }
}


