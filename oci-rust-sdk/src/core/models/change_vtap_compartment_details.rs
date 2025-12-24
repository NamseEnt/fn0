use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// These configuration details are used in the move operation when changing the compartment containing a virtual test access point (VTAP).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeVtapCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the destination compartment for the VTAP move.
    pub compartment_id: String,
}

/// Required fields for ChangeVtapCompartmentDetails
pub struct ChangeVtapCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the destination compartment for the VTAP move.
    pub compartment_id: String,
}

impl ChangeVtapCompartmentDetails {
    /// Create a new ChangeVtapCompartmentDetails with required fields
    pub fn new(required: ChangeVtapCompartmentDetailsRequired) -> Self {
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
