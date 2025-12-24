use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Details used when adding an IPv6 prefix to a subnet.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddSubnetIpv6CidrDetails {
    /// This field is not required and should only be specified when adding an IPv6 prefix to a subnet's IPv6 address space. See[IPv6 Addresses](https://docs.oracle.com/iaas/Content/Network/Concepts/ipv6.htm). <p> Example: {@code 2001:0db8:0123::/64}
    pub ipv6_cidr_block: String,
}

/// Required fields for AddSubnetIpv6CidrDetails
pub struct AddSubnetIpv6CidrDetailsRequired {
    /// This field is not required and should only be specified when adding an IPv6 prefix to a subnet's IPv6 address space. See[IPv6 Addresses](https://docs.oracle.com/iaas/Content/Network/Concepts/ipv6.htm). <p> Example: {@code 2001:0db8:0123::/64}
    pub ipv6_cidr_block: String,
}

impl AddSubnetIpv6CidrDetails {
    /// Create a new AddSubnetIpv6CidrDetails with required fields
    pub fn new(required: AddSubnetIpv6CidrDetailsRequired) -> Self {
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
