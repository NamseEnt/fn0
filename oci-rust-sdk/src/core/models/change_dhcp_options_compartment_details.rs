use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The configuration details for the move operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeDhcpOptionsCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the set of DHCP options to.
    pub compartment_id: String,
}

/// Required fields for ChangeDhcpOptionsCompartmentDetails
pub struct ChangeDhcpOptionsCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the set of DHCP options to.
    pub compartment_id: String,
}

impl ChangeDhcpOptionsCompartmentDetails {
    /// Create a new ChangeDhcpOptionsCompartmentDetails with required fields
    pub fn new(required: ChangeDhcpOptionsCompartmentDetailsRequired) -> Self {
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
