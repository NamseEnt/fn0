use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkDeleteVirtualCircuitPublicPrefixesRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the virtual circuit.
    pub virtual_circuit_id: String,

    /// Request with public prefixes to be deleted from the virtual circuit.
    pub bulk_delete_virtual_circuit_public_prefixes_details: BulkDeleteVirtualCircuitPublicPrefixesDetails,
}


/// Required fields for BulkDeleteVirtualCircuitPublicPrefixesRequest
pub struct BulkDeleteVirtualCircuitPublicPrefixesRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the virtual circuit.
    pub virtual_circuit_id: String,

    /// Request with public prefixes to be deleted from the virtual circuit.
    pub bulk_delete_virtual_circuit_public_prefixes_details: BulkDeleteVirtualCircuitPublicPrefixesDetails,
}

impl BulkDeleteVirtualCircuitPublicPrefixesRequest {
    /// Create a new BulkDeleteVirtualCircuitPublicPrefixesRequest with required fields
    pub fn new(required: BulkDeleteVirtualCircuitPublicPrefixesRequestRequired) -> Self {
        Self {
            virtual_circuit_id: required.virtual_circuit_id,

            bulk_delete_virtual_circuit_public_prefixes_details: required.bulk_delete_virtual_circuit_public_prefixes_details,
}
    }

    /// Set virtual_circuit_id
    pub fn set_virtual_circuit_id(mut self, value: String) -> Self {
        self.virtual_circuit_id = value;
        self
    }

    /// Set bulk_delete_virtual_circuit_public_prefixes_details
    pub fn set_bulk_delete_virtual_circuit_public_prefixes_details(mut self, value: BulkDeleteVirtualCircuitPublicPrefixesDetails) -> Self {
        self.bulk_delete_virtual_circuit_public_prefixes_details = value;
        self
    }
}


