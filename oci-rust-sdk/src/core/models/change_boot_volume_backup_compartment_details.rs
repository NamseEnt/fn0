use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Contains the details for the compartment to move the boot volume backup to.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeBootVolumeBackupCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the boot volume backup to.
    pub compartment_id: String,
}

/// Required fields for ChangeBootVolumeBackupCompartmentDetails
pub struct ChangeBootVolumeBackupCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the boot volume backup to.
    pub compartment_id: String,
}

impl ChangeBootVolumeBackupCompartmentDetails {
    /// Create a new ChangeBootVolumeBackupCompartmentDetails with required fields
    pub fn new(required: ChangeBootVolumeBackupCompartmentDetailsRequired) -> Self {
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
