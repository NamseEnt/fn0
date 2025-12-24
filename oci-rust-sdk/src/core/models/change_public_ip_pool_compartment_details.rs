use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The configuration details for the move operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePublicIpPoolCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the destination compartment for the public IP pool move.
    pub compartment_id: String,
}

/// Required fields for ChangePublicIpPoolCompartmentDetails
pub struct ChangePublicIpPoolCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the destination compartment for the public IP pool move.
    pub compartment_id: String,
}

impl ChangePublicIpPoolCompartmentDetails {
    /// Create a new ChangePublicIpPoolCompartmentDetails with required fields
    pub fn new(required: ChangePublicIpPoolCompartmentDetailsRequired) -> Self {
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
