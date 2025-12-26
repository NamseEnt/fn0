use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListVirtualCircuitPublicPrefixesRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the virtual circuit.
    pub virtual_circuit_id: String,

    /// A filter to only return resources that match the given verification state. <p> The state value is case-insensitive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_state: Option<String>,
}

/// Required fields for ListVirtualCircuitPublicPrefixesRequest
pub struct ListVirtualCircuitPublicPrefixesRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the virtual circuit.
    pub virtual_circuit_id: String,
}

impl ListVirtualCircuitPublicPrefixesRequest {
    /// Create a new ListVirtualCircuitPublicPrefixesRequest with required fields
    pub fn new(required: ListVirtualCircuitPublicPrefixesRequestRequired) -> Self {
        Self {
            virtual_circuit_id: required.virtual_circuit_id,

            verification_state: None,
        }
    }

    /// Set virtual_circuit_id
    pub fn set_virtual_circuit_id(mut self, value: String) -> Self {
        self.virtual_circuit_id = value;
        self
    }

    /// Set verification_state
    pub fn set_verification_state(mut self, value: Option<String>) -> Self {
        self.verification_state = value;
        self
    }

    /// Set verification_state (unwraps Option)
    pub fn with_verification_state(mut self, value: impl Into<String>) -> Self {
        self.verification_state = Some(value.into());
        self
    }
}
