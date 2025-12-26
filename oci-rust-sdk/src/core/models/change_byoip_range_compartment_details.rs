use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The configuration details for the move operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeByoipRangeCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the destination compartment for the BYOIP CIDR block move.
    pub compartment_id: String,
}

/// Required fields for ChangeByoipRangeCompartmentDetails
pub struct ChangeByoipRangeCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the destination compartment for the BYOIP CIDR block move.
    pub compartment_id: String,
}

impl ChangeByoipRangeCompartmentDetails {
    /// Create a new ChangeByoipRangeCompartmentDetails with required fields
    pub fn new(required: ChangeByoipRangeCompartmentDetailsRequired) -> Self {
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
