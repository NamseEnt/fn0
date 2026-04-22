use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The configuration details for the move operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeNatGatewayCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the NAT gateway to.
    pub compartment_id: String,
}

/// Required fields for ChangeNatGatewayCompartmentDetails
pub struct ChangeNatGatewayCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the NAT gateway to.
    pub compartment_id: String,
}

impl ChangeNatGatewayCompartmentDetails {
    /// Create a new ChangeNatGatewayCompartmentDetails with required fields
    pub fn new(required: ChangeNatGatewayCompartmentDetailsRequired) -> Self {
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
