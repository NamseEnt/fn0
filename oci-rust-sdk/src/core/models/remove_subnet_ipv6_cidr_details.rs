use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Details object for removing an IPv6 prefix from a subnet.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveSubnetIpv6CidrDetails {
    /// This field is not required and should only be specified when removing an IPv6 prefix from a subnet's IPv6 address space. See[IPv6 Addresses](https://docs.oracle.com/iaas/Content/Network/Concepts/ipv6.htm). <p> Example: {@code 2001:0db8:0123::/64}
    pub ipv6_cidr_block: String,
}

/// Required fields for RemoveSubnetIpv6CidrDetails
pub struct RemoveSubnetIpv6CidrDetailsRequired {
    /// This field is not required and should only be specified when removing an IPv6 prefix from a subnet's IPv6 address space. See[IPv6 Addresses](https://docs.oracle.com/iaas/Content/Network/Concepts/ipv6.htm). <p> Example: {@code 2001:0db8:0123::/64}
    pub ipv6_cidr_block: String,
}

impl RemoveSubnetIpv6CidrDetails {
    /// Create a new RemoveSubnetIpv6CidrDetails with required fields
    pub fn new(required: RemoveSubnetIpv6CidrDetailsRequired) -> Self {
        Self {
            ipv6_cidr_block: required.ipv6_cidr_block,
        }
    }

    /// Set ipv6_cidr_block
    pub fn set_ipv6_cidr_block(mut self, value: String) -> Self {
        self.ipv6_cidr_block = value;
        self
    }
}
