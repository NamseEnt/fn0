use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Contains the details for the compartment to move the boot volume to.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeBootVolumeCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the boot volume to.
    pub compartment_id: String,
}

/// Required fields for ChangeBootVolumeCompartmentDetails
pub struct ChangeBootVolumeCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the boot volume to.
    pub compartment_id: String,
}

impl ChangeBootVolumeCompartmentDetails {
    /// Create a new ChangeBootVolumeCompartmentDetails with required fields
    pub fn new(required: ChangeBootVolumeCompartmentDetailsRequired) -> Self {
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
