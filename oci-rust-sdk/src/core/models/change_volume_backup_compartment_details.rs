use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Contains the details for the compartment to move the volume backup to.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeVolumeBackupCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the volume backup to.
    pub compartment_id: String,
}

/// Required fields for ChangeVolumeBackupCompartmentDetails
pub struct ChangeVolumeBackupCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the volume backup to.
    pub compartment_id: String,
}

impl ChangeVolumeBackupCompartmentDetails {
    /// Create a new ChangeVolumeBackupCompartmentDetails with required fields
    pub fn new(required: ChangeVolumeBackupCompartmentDetailsRequired) -> Self {
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
