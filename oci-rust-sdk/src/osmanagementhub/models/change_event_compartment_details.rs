use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies which compartment to move the event to for the {@link #changeEventCompartment(ChangeEventCompartmentRequest) changeEventCompartment} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeEventCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the event to.
    pub compartment_id: String,
}

/// Required fields for ChangeEventCompartmentDetails
pub struct ChangeEventCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the event to.
    pub compartment_id: String,
}

impl ChangeEventCompartmentDetails {
    /// Create a new ChangeEventCompartmentDetails with required fields
    pub fn new(required: ChangeEventCompartmentDetailsRequired) -> Self {
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
