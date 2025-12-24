use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies the compartment to move the compute host to.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeComputeHostCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the compute host to.
    pub compartment_id: String,
}

/// Required fields for ChangeComputeHostCompartmentDetails
pub struct ChangeComputeHostCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the compute host to.
    pub compartment_id: String,
}

impl ChangeComputeHostCompartmentDetails {
    /// Create a new ChangeComputeHostCompartmentDetails with required fields
    pub fn new(required: ChangeComputeHostCompartmentDetailsRequired) -> Self {
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
