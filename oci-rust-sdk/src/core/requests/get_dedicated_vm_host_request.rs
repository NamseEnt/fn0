use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetDedicatedVmHostRequest {
    /// The OCID of the dedicated VM host.
    pub dedicated_vm_host_id: String,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}


/// Required fields for GetDedicatedVmHostRequest
pub struct GetDedicatedVmHostRequestRequired {
    /// The OCID of the dedicated VM host.
    pub dedicated_vm_host_id: String,
}

impl GetDedicatedVmHostRequest {
    /// Create a new GetDedicatedVmHostRequest with required fields
    pub fn new(required: GetDedicatedVmHostRequestRequired) -> Self {
        Self {
            dedicated_vm_host_id: required.dedicated_vm_host_id,

            opc_request_id: None,
}
    }

    /// Set dedicated_vm_host_id
    pub fn set_dedicated_vm_host_id(mut self, value: String) -> Self {
        self.dedicated_vm_host_id = value;
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


