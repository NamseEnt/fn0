use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Details object for updating the specified Ipv4 CIDR block of a Subnet.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifyIpv4SubnetCidrDetails {
    /// The Ipv4 CIDR IP address to update.
    pub ipv4_cidr_block: String,

    /// The new Ipv4 CIDR IP address.
    pub updated_ipv4_cidr_block: String,
}

/// Required fields for ModifyIpv4SubnetCidrDetails
pub struct ModifyIpv4SubnetCidrDetailsRequired {
    /// The Ipv4 CIDR IP address to update.
    pub ipv4_cidr_block: String,

    /// The new Ipv4 CIDR IP address.
    pub updated_ipv4_cidr_block: String,
}

impl ModifyIpv4SubnetCidrDetails {
    /// Create a new ModifyIpv4SubnetCidrDetails with required fields
    pub fn new(required: ModifyIpv4SubnetCidrDetailsRequired) -> Self {
        Self {
            ipv4_cidr_block: required.ipv4_cidr_block,

            updated_ipv4_cidr_block: required.updated_ipv4_cidr_block,
        }
    }

    /// Set ipv4_cidr_block
    pub fn set_ipv4_cidr_block(mut self, value: String) -> Self {
        self.ipv4_cidr_block = value;
        self
    }

    /// Set updated_ipv4_cidr_block
    pub fn set_updated_ipv4_cidr_block(mut self, value: String) -> Self {
        self.updated_ipv4_cidr_block = value;
        self
    }
}
