use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Details used to add a CIDR block to a VCN.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddVcnCidrDetails {
    /// The CIDR block to add.
    pub cidr_block: String,
}

/// Required fields for AddVcnCidrDetails
pub struct AddVcnCidrDetailsRequired {
    /// The CIDR block to add.
    pub cidr_block: String,
}

impl AddVcnCidrDetails {
    /// Create a new AddVcnCidrDetails with required fields
    pub fn new(required: AddVcnCidrDetailsRequired) -> Self {
        Self {
            cidr_block: required.cidr_block,
        }
    }

    /// Set cidr_block
    pub fn set_cidr_block(mut self, value: String) -> Self {
        self.cidr_block = value;
        self
    }
}
