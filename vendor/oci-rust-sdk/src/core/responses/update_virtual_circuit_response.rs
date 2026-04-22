use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVirtualCircuitResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.VirtualCircuit instance.
    pub virtual_circuit: VirtualCircuit,
}

/// Required fields for UpdateVirtualCircuitResponse
pub struct UpdateVirtualCircuitResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.VirtualCircuit instance.
    pub virtual_circuit: VirtualCircuit,
}

impl UpdateVirtualCircuitResponse {
    /// Create a new UpdateVirtualCircuitResponse with required fields
    pub fn new(required: UpdateVirtualCircuitResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            virtual_circuit: required.virtual_circuit,
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

    /// Set virtual_circuit
    pub fn set_virtual_circuit(mut self, value: VirtualCircuit) -> Self {
        self.virtual_circuit = value;
        self
    }
}
