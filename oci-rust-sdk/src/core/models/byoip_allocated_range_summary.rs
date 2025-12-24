use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A summary of CIDR block subranges that are currently allocated to an IP pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ByoipAllocatedRangeSummary {
    /// The BYOIP CIDR block range or subrange allocated to an IP pool. This could be all or part of a BYOIP CIDR block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cidr_block: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the IP pool containing the CIDR block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_ip_pool_id: Option<String>,
}

impl ByoipAllocatedRangeSummary {
    /// Create a new ByoipAllocatedRangeSummary
    pub fn new() -> Self {
        Self {
            cidr_block: None,

            public_ip_pool_id: None,
        }
    }

    /// Set cidr_block
    pub fn set_cidr_block(mut self, value: Option<String>) -> Self {
        self.cidr_block = value;
        self
    }

    /// Set public_ip_pool_id
    pub fn set_public_ip_pool_id(mut self, value: Option<String>) -> Self {
        self.public_ip_pool_id = value;
        self
    }

    /// Set cidr_block (unwraps Option)
    pub fn with_cidr_block(mut self, value: impl Into<String>) -> Self {
        self.cidr_block = Some(value.into());
        self
    }

    /// Set public_ip_pool_id (unwraps Option)
    pub fn with_public_ip_pool_id(mut self, value: impl Into<String>) -> Self {
        self.public_ip_pool_id = Some(value.into());
        self
    }
}

impl Default for ByoipAllocatedRangeSummary {
    fn default() -> Self {
        Self::new()
    }
}
