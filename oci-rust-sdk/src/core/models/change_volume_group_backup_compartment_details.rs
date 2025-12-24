use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Contains the details for the compartment to move the volume group backup to.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeVolumeGroupBackupCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the volume group backup to.
    pub compartment_id: String,
}

/// Required fields for ChangeVolumeGroupBackupCompartmentDetails
pub struct ChangeVolumeGroupBackupCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the volume group backup to.
    pub compartment_id: String,
}

impl ChangeVolumeGroupBackupCompartmentDetails {
    /// Create a new ChangeVolumeGroupBackupCompartmentDetails with required fields
    pub fn new(required: ChangeVolumeGroupBackupCompartmentDetailsRequired) -> Self {
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
