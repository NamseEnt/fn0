use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Details to assign an IPv6 subnet prefix and IPv6 address on VNIC creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ipv6AddressIpv6SubnetCidrPairDetails {
    /// The IPv6 prefix allocated to the subnet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_subnet_cidr: Option<String>,

    /// An IPv6 address of your choice. Must be an available IPv6 address within the subnet's prefix. If an IPv6 address is not provided: - Oracle will automatically assign an IPv6 address from the subnet's IPv6 prefix if and only if there is only one IPv6 prefix on the subnet. - Oracle will automatically assign an IPv6 address from the subnet's IPv6 Oracle GUA prefix if it exists on the subnet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_address: Option<String>,
}

impl Ipv6AddressIpv6SubnetCidrPairDetails {
    /// Create a new Ipv6AddressIpv6SubnetCidrPairDetails
    pub fn new() -> Self {
        Self {
            ipv6_subnet_cidr: None,

            ipv6_address: None,
        }
    }

    /// Set ipv6_subnet_cidr
    pub fn set_ipv6_subnet_cidr(mut self, value: Option<String>) -> Self {
        self.ipv6_subnet_cidr = value;
        self
    }

    /// Set ipv6_address
    pub fn set_ipv6_address(mut self, value: Option<String>) -> Self {
        self.ipv6_address = value;
        self
    }

    /// Set ipv6_subnet_cidr (unwraps Option)
    pub fn with_ipv6_subnet_cidr(mut self, value: impl Into<String>) -> Self {
        self.ipv6_subnet_cidr = Some(value.into());
        self
    }

    /// Set ipv6_address (unwraps Option)
    pub fn with_ipv6_address(mut self, value: impl Into<String>) -> Self {
        self.ipv6_address = Some(value.into());
        self
    }
}

impl Default for Ipv6AddressIpv6SubnetCidrPairDetails {
    fn default() -> Self {
        Self::new()
    }
}
