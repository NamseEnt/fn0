use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Contains the details for the compartment to move the volume group to.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeVolumeGroupCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the volume group to.
    pub compartment_id: String,
}

/// Required fields for ChangeVolumeGroupCompartmentDetails
pub struct ChangeVolumeGroupCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the volume group to.
    pub compartment_id: String,
}

impl ChangeVolumeGroupCompartmentDetails {
    /// Create a new ChangeVolumeGroupCompartmentDetails with required fields
    pub fn new(required: ChangeVolumeGroupCompartmentDetailsRequired) -> Self {
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
