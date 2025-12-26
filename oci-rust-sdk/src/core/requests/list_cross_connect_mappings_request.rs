use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListCrossConnectMappingsRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the virtual circuit.
    pub virtual_circuit_id: String,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}

/// Required fields for ListCrossConnectMappingsRequest
pub struct ListCrossConnectMappingsRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the virtual circuit.
    pub virtual_circuit_id: String,
}

impl ListCrossConnectMappingsRequest {
    /// Create a new ListCrossConnectMappingsRequest with required fields
    pub fn new(required: ListCrossConnectMappingsRequestRequired) -> Self {
        Self {
            virtual_circuit_id: required.virtual_circuit_id,

            opc_request_id: None,
        }
    }

    /// Set virtual_circuit_id
    pub fn set_virtual_circuit_id(mut self, value: String) -> Self {
        self.virtual_circuit_id = value;
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
