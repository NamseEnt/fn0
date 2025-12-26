use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The configuration details for the move operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeIPSecConnectionCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the IPSec connection to.
    pub compartment_id: String,
}

/// Required fields for ChangeIPSecConnectionCompartmentDetails
pub struct ChangeIPSecConnectionCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the IPSec connection to.
    pub compartment_id: String,
}

impl ChangeIPSecConnectionCompartmentDetails {
    /// Create a new ChangeIPSecConnectionCompartmentDetails with required fields
    pub fn new(required: ChangeIPSecConnectionCompartmentDetailsRequired) -> Self {
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
