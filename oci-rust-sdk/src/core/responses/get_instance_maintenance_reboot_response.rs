use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetInstanceMaintenanceRebootResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.InstanceMaintenanceReboot instance.
    pub instance_maintenance_reboot: InstanceMaintenanceReboot,
}


/// Required fields for GetInstanceMaintenanceRebootResponse
pub struct GetInstanceMaintenanceRebootResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.InstanceMaintenanceReboot instance.
    pub instance_maintenance_reboot: InstanceMaintenanceReboot,
}

impl GetInstanceMaintenanceRebootResponse {
    /// Create a new GetInstanceMaintenanceRebootResponse with required fields
    pub fn new(required: GetInstanceMaintenanceRebootResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,

            instance_maintenance_reboot: required.instance_maintenance_reboot,
}
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set instance_maintenance_reboot
    pub fn set_instance_maintenance_reboot(mut self, value: InstanceMaintenanceReboot) -> Self {
        self.instance_maintenance_reboot = value;
        self
    }
}


