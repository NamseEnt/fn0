use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Details for removing a CIDR block from a VCN.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveVcnCidrDetails {
    /// The CIDR block to remove.
    pub cidr_block: String,
}

/// Required fields for RemoveVcnCidrDetails
pub struct RemoveVcnCidrDetailsRequired {
    /// The CIDR block to remove.
    pub cidr_block: String,
}

impl RemoveVcnCidrDetails {
    /// Create a new RemoveVcnCidrDetails with required fields
    pub fn new(required: RemoveVcnCidrDetailsRequired) -> Self {
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
