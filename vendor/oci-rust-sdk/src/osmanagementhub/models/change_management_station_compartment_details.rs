use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to move a management station to a different compartment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeManagementStationCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the management station to.
    pub compartment_id: String,
}

/// Required fields for ChangeManagementStationCompartmentDetails
pub struct ChangeManagementStationCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the management station to.
    pub compartment_id: String,
}

impl ChangeManagementStationCompartmentDetails {
    /// Create a new ChangeManagementStationCompartmentDetails with required fields
    pub fn new(required: ChangeManagementStationCompartmentDetailsRequired) -> Self {
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
