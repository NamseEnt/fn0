use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkDeleteVirtualCircuitPublicPrefixesDetails {
    /// The public IP prefixes (CIDRs) to remove from the public virtual circuit.
    pub public_prefixes: Vec<DeleteVirtualCircuitPublicPrefixDetails>,
}

/// Required fields for BulkDeleteVirtualCircuitPublicPrefixesDetails
pub struct BulkDeleteVirtualCircuitPublicPrefixesDetailsRequired {
    /// The public IP prefixes (CIDRs) to remove from the public virtual circuit.
    pub public_prefixes: Vec<DeleteVirtualCircuitPublicPrefixDetails>,
}

impl BulkDeleteVirtualCircuitPublicPrefixesDetails {
    /// Create a new BulkDeleteVirtualCircuitPublicPrefixesDetails with required fields
    pub fn new(required: BulkDeleteVirtualCircuitPublicPrefixesDetailsRequired) -> Self {
        Self {
            public_prefixes: required.public_prefixes,
        }
    }

    /// Set public_prefixes
    pub fn set_public_prefixes(
        mut self,
        value: Vec<DeleteVirtualCircuitPublicPrefixDetails>,
    ) -> Self {
        self.public_prefixes = value;
        self
    }
}
