use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Details object for removing an IPv4 prefix from a subnet.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveIpv4SubnetCidrDetails {
    /// This field should only be specified when removing an IPv4 prefix from a subnet's IPv4 address space. The CIDR must maintain the following rules - <p> a. The CIDR block is valid and correctly formatted. b. The CIDR range is within one of the parent VCN ranges. c. The CIDR range to be removed is already present in the list of ipv4CidrBlocks <p> Example: {@code 10.0.1.0/24}
    pub ipv4_cidr_block: String,
}

/// Required fields for RemoveIpv4SubnetCidrDetails
pub struct RemoveIpv4SubnetCidrDetailsRequired {
    /// This field should only be specified when removing an IPv4 prefix from a subnet's IPv4 address space. The CIDR must maintain the following rules - <p> a. The CIDR block is valid and correctly formatted. b. The CIDR range is within one of the parent VCN ranges. c. The CIDR range to be removed is already present in the list of ipv4CidrBlocks <p> Example: {@code 10.0.1.0/24}
    pub ipv4_cidr_block: String,
}

impl RemoveIpv4SubnetCidrDetails {
    /// Create a new RemoveIpv4SubnetCidrDetails with required fields
    pub fn new(required: RemoveIpv4SubnetCidrDetailsRequired) -> Self {
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
