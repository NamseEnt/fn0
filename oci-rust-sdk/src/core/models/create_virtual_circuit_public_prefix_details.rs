use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVirtualCircuitPublicPrefixDetails {
    /// An individual public IP prefix (CIDR) to add to the public virtual circuit. All prefix sizes are allowed.
    pub cidr_block: String,
}

/// Required fields for CreateVirtualCircuitPublicPrefixDetails
pub struct CreateVirtualCircuitPublicPrefixDetailsRequired {
    /// An individual public IP prefix (CIDR) to add to the public virtual circuit. All prefix sizes are allowed.
    pub cidr_block: String,
}

impl CreateVirtualCircuitPublicPrefixDetails {
    /// Create a new CreateVirtualCircuitPublicPrefixDetails with required fields
    pub fn new(required: CreateVirtualCircuitPublicPrefixDetailsRequired) -> Self {
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
