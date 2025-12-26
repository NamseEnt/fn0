use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies the compartment to move the compute GPU memory fabric to.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeComputeGpuMemoryFabricCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the compute GPU memory fabric to.
    pub compartment_id: String,
}

/// Required fields for ChangeComputeGpuMemoryFabricCompartmentDetails
pub struct ChangeComputeGpuMemoryFabricCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the compute GPU memory fabric to.
    pub compartment_id: String,
}

impl ChangeComputeGpuMemoryFabricCompartmentDetails {
    /// Create a new ChangeComputeGpuMemoryFabricCompartmentDetails with required fields
    pub fn new(required: ChangeComputeGpuMemoryFabricCompartmentDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }
}
