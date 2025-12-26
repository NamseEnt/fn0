use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkAddVirtualCircuitPublicPrefixesRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the virtual circuit.
    pub virtual_circuit_id: String,

    /// Request with publix prefixes to be added to the virtual circuit
    pub bulk_add_virtual_circuit_public_prefixes_details:
        BulkAddVirtualCircuitPublicPrefixesDetails,
}

/// Required fields for BulkAddVirtualCircuitPublicPrefixesRequest
pub struct BulkAddVirtualCircuitPublicPrefixesRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the virtual circuit.
    pub virtual_circuit_id: String,

    /// Request with publix prefixes to be added to the virtual circuit
    pub bulk_add_virtual_circuit_public_prefixes_details:
        BulkAddVirtualCircuitPublicPrefixesDetails,
}

impl BulkAddVirtualCircuitPublicPrefixesRequest {
    /// Create a new BulkAddVirtualCircuitPublicPrefixesRequest with required fields
    pub fn new(required: BulkAddVirtualCircuitPublicPrefixesRequestRequired) -> Self {
        Self {
            virtual_circuit_id: required.virtual_circuit_id,

            bulk_add_virtual_circuit_public_prefixes_details: required
                .bulk_add_virtual_circuit_public_prefixes_details,
        }
    }

    /// Set virtual_circuit_id
    pub fn set_virtual_circuit_id(mut self, value: String) -> Self {
        self.virtual_circuit_id = value;
        self
    }

    /// Set bulk_add_virtual_circuit_public_prefixes_details
    pub fn set_bulk_add_virtual_circuit_public_prefixes_details(
        mut self,
        value: BulkAddVirtualCircuitPublicPrefixesDetails,
    ) -> Self {
        self.bulk_add_virtual_circuit_public_prefixes_details = value;
        self
    }
}
