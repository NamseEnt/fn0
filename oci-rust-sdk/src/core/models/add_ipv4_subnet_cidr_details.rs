use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Details used when adding an IPv4 prefix to a subnet.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddIpv4SubnetCidrDetails {
    /// The CIDR IP address range of the subnet. The CIDR must maintain the following rules - <p> a. The CIDR block is valid and correctly formatted. b. The new range is within one of the parent VCN ranges. <p> Example: {@code 10.0.1.0/24}
    pub ipv4_cidr_block: String,
}

/// Required fields for AddIpv4SubnetCidrDetails
pub struct AddIpv4SubnetCidrDetailsRequired {
    /// The CIDR IP address range of the subnet. The CIDR must maintain the following rules - <p> a. The CIDR block is valid and correctly formatted. b. The new range is within one of the parent VCN ranges. <p> Example: {@code 10.0.1.0/24}
    pub ipv4_cidr_block: String,
}

impl AddIpv4SubnetCidrDetails {
    /// Create a new AddIpv4SubnetCidrDetails with required fields
    pub fn new(required: AddIpv4SubnetCidrDetailsRequired) -> Self {
        Self {
            ipv4_cidr_block: required.ipv4_cidr_block,
        }
    }

    /// Set ipv4_cidr_block
    pub fn set_ipv4_cidr_block(mut self, value: String) -> Self {
        self.ipv4_cidr_block = value;
        self
    }
}
