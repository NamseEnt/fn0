use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies the compartment to move the compute GPU memory cluster to.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeComputeGpuMemoryClusterCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the compute GPU memory cluster to.
    pub compartment_id: String,
}

/// Required fields for ChangeComputeGpuMemoryClusterCompartmentDetails
pub struct ChangeComputeGpuMemoryClusterCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the compute GPU memory cluster to.
    pub compartment_id: String,
}

impl ChangeComputeGpuMemoryClusterCompartmentDetails {
    /// Create a new ChangeComputeGpuMemoryClusterCompartmentDetails with required fields
    pub fn new(required: ChangeComputeGpuMemoryClusterCompartmentDetailsRequired) -> Self {
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
