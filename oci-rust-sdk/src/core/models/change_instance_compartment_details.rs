use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The configuration details for the move operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeInstanceCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the instance to.
    pub compartment_id: String,
}

/// Required fields for ChangeInstanceCompartmentDetails
pub struct ChangeInstanceCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the instance to.
    pub compartment_id: String,
}

impl ChangeInstanceCompartmentDetails {
    /// Create a new ChangeInstanceCompartmentDetails with required fields
    pub fn new(required: ChangeInstanceCompartmentDetailsRequired) -> Self {
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
