use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeNetworkSecurityGroupCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the network security group to.
    pub compartment_id: String,
}

/// Required fields for ChangeNetworkSecurityGroupCompartmentDetails
pub struct ChangeNetworkSecurityGroupCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the network security group to.
    pub compartment_id: String,
}

impl ChangeNetworkSecurityGroupCompartmentDetails {
    /// Create a new ChangeNetworkSecurityGroupCompartmentDetails with required fields
    pub fn new(required: ChangeNetworkSecurityGroupCompartmentDetailsRequired) -> Self {
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
