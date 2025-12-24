use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Optional. Used to specify from which subnet prefixes an IPv6 address should be allocated, or to assign valid available IPv6 addresses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstancePoolPlacementIpv6AddressIpv6SubnetCidrDetails {
    /// Optional. Used to disambiguate which subnet prefix should be used to create an IPv6 allocation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_subnet_cidr: Option<String>,
}

impl InstancePoolPlacementIpv6AddressIpv6SubnetCidrDetails {
    /// Create a new InstancePoolPlacementIpv6AddressIpv6SubnetCidrDetails
    pub fn new() -> Self {
        Self {
            ipv6_subnet_cidr: None,
        }
    }

    /// Set ipv6_subnet_cidr
    pub fn set_ipv6_subnet_cidr(mut self, value: Option<String>) -> Self {
        self.ipv6_subnet_cidr = value;
        self
    }

    /// Set ipv6_subnet_cidr (unwraps Option)
    pub fn with_ipv6_subnet_cidr(mut self, value: impl Into<String>) -> Self {
        self.ipv6_subnet_cidr = Some(value.into());
        self
    }
}

impl Default for InstancePoolPlacementIpv6AddressIpv6SubnetCidrDetails {
    fn default() -> Self {
        Self::new()
    }
}
