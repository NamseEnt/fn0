use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The configuration details for the move operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeCrossConnectGroupCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the cross-connect group to.
    pub compartment_id: String,
}

/// Required fields for ChangeCrossConnectGroupCompartmentDetails
pub struct ChangeCrossConnectGroupCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the cross-connect group to.
    pub compartment_id: String,
}

impl ChangeCrossConnectGroupCompartmentDetails {
    /// Create a new ChangeCrossConnectGroupCompartmentDetails with required fields
    pub fn new(required: ChangeCrossConnectGroupCompartmentDetailsRequired) -> Self {
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
