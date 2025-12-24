use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to move the managed instance group to another compartment within the same tenancy. For information about moving resources between compartments, see [Moving Resources to a Different Compartment](https://docs.oracle.com/iaas/Content/Identity/Tasks/managingcompartments.htm#moveRes).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeManagedInstanceGroupCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the group to.
    pub compartment_id: String,
}

/// Required fields for ChangeManagedInstanceGroupCompartmentDetails
pub struct ChangeManagedInstanceGroupCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the group to.
    pub compartment_id: String,
}

impl ChangeManagedInstanceGroupCompartmentDetails {
    /// Create a new ChangeManagedInstanceGroupCompartmentDetails with required fields
    pub fn new(required: ChangeManagedInstanceGroupCompartmentDetailsRequired) -> Self {
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
