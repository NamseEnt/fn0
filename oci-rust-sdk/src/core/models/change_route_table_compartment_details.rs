use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The configuration details for the move operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeRouteTableCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the route table to.
    pub compartment_id: String,
}

/// Required fields for ChangeRouteTableCompartmentDetails
pub struct ChangeRouteTableCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the route table to.
    pub compartment_id: String,
}

impl ChangeRouteTableCompartmentDetails {
    /// Create a new ChangeRouteTableCompartmentDetails with required fields
    pub fn new(required: ChangeRouteTableCompartmentDetailsRequired) -> Self {
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
