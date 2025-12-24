use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Details for updating a CIDR block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifyVcnCidrDetails {
    /// The CIDR IP address to update.
    pub original_cidr_block: String,

    /// The new CIDR IP address.
    pub new_cidr_block: String,
}

/// Required fields for ModifyVcnCidrDetails
pub struct ModifyVcnCidrDetailsRequired {
    /// The CIDR IP address to update.
    pub original_cidr_block: String,

    /// The new CIDR IP address.
    pub new_cidr_block: String,
}

impl ModifyVcnCidrDetails {
    /// Create a new ModifyVcnCidrDetails with required fields
    pub fn new(required: ModifyVcnCidrDetailsRequired) -> Self {
        Self {
            original_cidr_block: required.original_cidr_block,

            new_cidr_block: required.new_cidr_block,
        }
    }

    /// Set original_cidr_block
    pub fn set_original_cidr_block(mut self, value: String) -> Self {
        self.original_cidr_block = value;
        self
    }

    /// Set new_cidr_block
    pub fn set_new_cidr_block(mut self, value: String) -> Self {
        self.new_cidr_block = value;
        self
    }
}
