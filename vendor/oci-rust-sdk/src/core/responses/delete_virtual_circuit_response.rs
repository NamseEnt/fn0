use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteVirtualCircuitResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,
}

/// Required fields for DeleteVirtualCircuitResponse
pub struct DeleteVirtualCircuitResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,
}

impl DeleteVirtualCircuitResponse {
    /// Create a new DeleteVirtualCircuitResponse with required fields
    pub fn new(required: DeleteVirtualCircuitResponseRequired) -> Self {
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
