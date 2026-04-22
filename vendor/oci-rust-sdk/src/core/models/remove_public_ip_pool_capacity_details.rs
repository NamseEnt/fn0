use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The information needed to remove capacity from a public IP pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemovePublicIpPoolCapacityDetails {
    /// The CIDR block to remove from the  public IP pool. Example: {@code 10.0.1.0/24}
    pub cidr_block: String,
}

/// Required fields for RemovePublicIpPoolCapacityDetails
pub struct RemovePublicIpPoolCapacityDetailsRequired {
    /// The CIDR block to remove from the  public IP pool. Example: {@code 10.0.1.0/24}
    pub cidr_block: String,
}

impl RemovePublicIpPoolCapacityDetails {
    /// Create a new RemovePublicIpPoolCapacityDetails with required fields
    pub fn new(required: RemovePublicIpPoolCapacityDetailsRequired) -> Self {
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
