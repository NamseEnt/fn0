use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Contains the details for the compartment to move the volume to.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeVolumeCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the volume to.
    pub compartment_id: String,
}

/// Required fields for ChangeVolumeCompartmentDetails
pub struct ChangeVolumeCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the volume to.
    pub compartment_id: String,
}

impl ChangeVolumeCompartmentDetails {
    /// Create a new ChangeVolumeCompartmentDetails with required fields
    pub fn new(required: ChangeVolumeCompartmentDetailsRequired) -> Self {
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
