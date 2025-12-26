use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteVirtualCircuitRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the virtual circuit.
    pub virtual_circuit_id: String,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}

/// Required fields for DeleteVirtualCircuitRequest
pub struct DeleteVirtualCircuitRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the virtual circuit.
    pub virtual_circuit_id: String,
}

impl DeleteVirtualCircuitRequest {
    /// Create a new DeleteVirtualCircuitRequest with required fields
    pub fn new(required: DeleteVirtualCircuitRequestRequired) -> Self {
        Self {
            virtual_circuit_id: required.virtual_circuit_id,

            if_match: None,
        }
    }

    /// Set virtual_circuit_id
    pub fn set_virtual_circuit_id(mut self, value: String) -> Self {
        self.virtual_circuit_id = value;
        self
    }

    /// Set if_match
    pub fn set_if_match(mut self, value: Option<String>) -> Self {
        self.if_match = value;
        self
    }

    /// Set if_match (unwraps Option)
    pub fn with_if_match(mut self, value: impl Into<String>) -> Self {
        self.if_match = Some(value.into());
        self
    }
}
