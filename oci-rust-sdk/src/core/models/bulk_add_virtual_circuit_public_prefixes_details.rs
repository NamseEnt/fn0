use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkAddVirtualCircuitPublicPrefixesDetails {
    /// The public IP prefixes (CIDRs) to add to the public virtual circuit.
    pub public_prefixes: Vec<CreateVirtualCircuitPublicPrefixDetails>,
}

/// Required fields for BulkAddVirtualCircuitPublicPrefixesDetails
pub struct BulkAddVirtualCircuitPublicPrefixesDetailsRequired {
    /// The public IP prefixes (CIDRs) to add to the public virtual circuit.
    pub public_prefixes: Vec<CreateVirtualCircuitPublicPrefixDetails>,
}

impl BulkAddVirtualCircuitPublicPrefixesDetails {
    /// Create a new BulkAddVirtualCircuitPublicPrefixesDetails with required fields
    pub fn new(required: BulkAddVirtualCircuitPublicPrefixesDetailsRequired) -> Self {
        Self {
            public_prefixes: required.public_prefixes,
        }
    }

    /// Set public_prefixes
    pub fn set_public_prefixes(
        mut self,
        value: Vec<CreateVirtualCircuitPublicPrefixDetails>,
    ) -> Self {
        self.public_prefixes = value;
        self
    }
}
