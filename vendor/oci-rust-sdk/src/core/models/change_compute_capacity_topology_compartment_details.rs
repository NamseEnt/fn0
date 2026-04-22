use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies the compartment to move the compute capacity topology to.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeComputeCapacityTopologyCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the compute capacity topology to.
    pub compartment_id: String,
}

/// Required fields for ChangeComputeCapacityTopologyCompartmentDetails
pub struct ChangeComputeCapacityTopologyCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the compute capacity topology to.
    pub compartment_id: String,
}

impl ChangeComputeCapacityTopologyCompartmentDetails {
    /// Create a new ChangeComputeCapacityTopologyCompartmentDetails with required fields
    pub fn new(required: ChangeComputeCapacityTopologyCompartmentDetailsRequired) -> Self {
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
