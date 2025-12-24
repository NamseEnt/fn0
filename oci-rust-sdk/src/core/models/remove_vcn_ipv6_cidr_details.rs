use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Details used when removing ULA or private IPv6 prefix or an IPv6 GUA assigned by Oracle or BYOIPv6 prefix. You can only remove one of these per request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveVcnIpv6CidrDetails {
    /// This field is not required and should only be specified when removing ULA or private IPv6 prefix or an IPv6 GUA assigned by Oracle or BYOIPv6 prefix from a VCN's IPv6 address space. See[IPv6 Addresses](https://docs.oracle.com/iaas/Content/Network/Concepts/ipv6.htm). <p> Example: {@code 2001:0db8:0123::/56}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_cidr_block: Option<String>,
}

impl RemoveVcnIpv6CidrDetails {
    /// Create a new RemoveVcnIpv6CidrDetails
    pub fn new() -> Self {
        Self {
            ipv6_cidr_block: None,
        }
    }

    /// Set ipv6_cidr_block
    pub fn set_ipv6_cidr_block(mut self, value: Option<String>) -> Self {
        self.ipv6_cidr_block = value;
        self
    }

    /// Set ipv6_cidr_block (unwraps Option)
    pub fn with_ipv6_cidr_block(mut self, value: impl Into<String>) -> Self {
        self.ipv6_cidr_block = Some(value.into());
        self
    }
}

impl Default for RemoveVcnIpv6CidrDetails {
    fn default() -> Self {
        Self::new()
    }
}
