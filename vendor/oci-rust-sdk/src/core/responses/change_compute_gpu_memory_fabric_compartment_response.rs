use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeComputeGpuMemoryFabricCompartmentResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,
}

/// Required fields for ChangeComputeGpuMemoryFabricCompartmentResponse
pub struct ChangeComputeGpuMemoryFabricCompartmentResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,
}

impl ChangeComputeGpuMemoryFabricCompartmentResponse {
    /// Create a new ChangeComputeGpuMemoryFabricCompartmentResponse with required fields
    pub fn new(required: ChangeComputeGpuMemoryFabricCompartmentResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,
        }
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }
}
