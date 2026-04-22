use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVirtualCircuitRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the virtual circuit.
    pub virtual_circuit_id: String,
}

/// Required fields for GetVirtualCircuitRequest
pub struct GetVirtualCircuitRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the virtual circuit.
    pub virtual_circuit_id: String,
}

impl GetVirtualCircuitRequest {
    /// Create a new GetVirtualCircuitRequest with required fields
    pub fn new(required: GetVirtualCircuitRequestRequired) -> Self {
        Self {
            virtual_circuit_id: required.virtual_circuit_id,
        }
    }

    /// Set virtual_circuit_id
    pub fn set_virtual_circuit_id(mut self, value: String) -> Self {
        self.virtual_circuit_id = value;
        self
    }
}
