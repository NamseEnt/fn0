use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Optional. Used to specify from which subnet prefixes an IPv6 address should be allocated, or to assign valid available IPv6 addresses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationIpv6AddressIpv6SubnetCidrPairDetails {
    /// Optional. Used to disambiguate which subnet prefix should be used to create an IPv6 allocation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_subnet_cidr: Option<String>,

    /// Optional. An available IPv6 address of your subnet from a valid IPv6 prefix on the subnet (otherwise the IP address is automatically assigned).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_address: Option<String>,
}

impl InstanceConfigurationIpv6AddressIpv6SubnetCidrPairDetails {
    /// Create a new InstanceConfigurationIpv6AddressIpv6SubnetCidrPairDetails
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

impl Default for InstanceConfigurationIpv6AddressIpv6SubnetCidrPairDetails {
    fn default() -> Self {
        Self::new()
    }
}
