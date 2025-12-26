use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The configuration details for the move operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePublicIpCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the public IP to.
    pub compartment_id: String,
}

/// Required fields for ChangePublicIpCompartmentDetails
pub struct ChangePublicIpCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the public IP to.
    pub compartment_id: String,
}

impl ChangePublicIpCompartmentDetails {
    /// Create a new ChangePublicIpCompartmentDetails with required fields
    pub fn new(required: ChangePublicIpCompartmentDetailsRequired) -> Self {
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
