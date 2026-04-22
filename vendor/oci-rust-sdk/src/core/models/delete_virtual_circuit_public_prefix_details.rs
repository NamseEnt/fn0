use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteVirtualCircuitPublicPrefixDetails {
    /// An individual public IP prefix (CIDR) to remove from the public virtual circuit.
    pub cidr_block: String,
}

/// Required fields for DeleteVirtualCircuitPublicPrefixDetails
pub struct DeleteVirtualCircuitPublicPrefixDetailsRequired {
    /// An individual public IP prefix (CIDR) to remove from the public virtual circuit.
    pub cidr_block: String,
}

impl DeleteVirtualCircuitPublicPrefixDetails {
    /// Create a new DeleteVirtualCircuitPublicPrefixDetails with required fields
    pub fn new(required: DeleteVirtualCircuitPublicPrefixDetailsRequired) -> Self {
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
