use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies the compartment to move the dedicated virtual machine host to.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeDedicatedVmHostCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the dedicated virtual machine host to.
    pub compartment_id: String,
}

/// Required fields for ChangeDedicatedVmHostCompartmentDetails
pub struct ChangeDedicatedVmHostCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the dedicated virtual machine host to.
    pub compartment_id: String,
}

impl ChangeDedicatedVmHostCompartmentDetails {
    /// Create a new ChangeDedicatedVmHostCompartmentDetails with required fields
    pub fn new(required: ChangeDedicatedVmHostCompartmentDetailsRequired) -> Self {
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
