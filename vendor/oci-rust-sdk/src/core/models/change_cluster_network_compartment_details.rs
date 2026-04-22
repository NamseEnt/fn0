use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The configuration details for the move operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeClusterNetworkCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment into which the resource should be moved.
    pub compartment_id: String,
}

/// Required fields for ChangeClusterNetworkCompartmentDetails
pub struct ChangeClusterNetworkCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment into which the resource should be moved.
    pub compartment_id: String,
}

impl ChangeClusterNetworkCompartmentDetails {
    /// Create a new ChangeClusterNetworkCompartmentDetails with required fields
    pub fn new(required: ChangeClusterNetworkCompartmentDetailsRequired) -> Self {
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
