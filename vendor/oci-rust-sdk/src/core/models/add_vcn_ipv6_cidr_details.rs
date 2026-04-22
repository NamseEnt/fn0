use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Details used when adding a ULA or private IPv6 prefix or an IPv6 GUA assigned by Oracle or a BYOIPv6 prefix. You can add only one of these per request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddVcnIpv6CidrDetails {
    /// This field is not required and should only be specified if a ULA or private IPv6 prefix is desired for VCN's private IP address space. See[IPv6 Addresses](https://docs.oracle.com/iaas/Content/Network/Concepts/ipv6.htm). <p> Example: {@code 2001:0db8:0123::/48} or {@code fd00:1000:0:1::/64}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_private_cidr_block: Option<String>,

    /// Indicates whether Oracle will allocate an IPv6 GUA. Only one prefix of /56 size can be allocated by Oracle as a GUA.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_oracle_gua_allocation_enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub byoipv6_cidr_detail: Option<Byoipv6CidrDetails>,
}

impl AddVcnIpv6CidrDetails {
    /// Create a new AddVcnIpv6CidrDetails
    pub fn new() -> Self {
        Self {
            ipv6_private_cidr_block: None,

            is_oracle_gua_allocation_enabled: None,

            byoipv6_cidr_detail: None,
        }
    }

    /// Set ipv6_private_cidr_block
    pub fn set_ipv6_private_cidr_block(mut self, value: Option<String>) -> Self {
        self.ipv6_private_cidr_block = value;
        self
    }

    /// Set is_oracle_gua_allocation_enabled
    pub fn set_is_oracle_gua_allocation_enabled(mut self, value: Option<bool>) -> Self {
        self.is_oracle_gua_allocation_enabled = value;
        self
    }

    /// Set byoipv6_cidr_detail
    pub fn set_byoipv6_cidr_detail(mut self, value: Option<Byoipv6CidrDetails>) -> Self {
        self.byoipv6_cidr_detail = value;
        self
    }

    /// Set ipv6_private_cidr_block (unwraps Option)
    pub fn with_ipv6_private_cidr_block(mut self, value: impl Into<String>) -> Self {
        self.ipv6_private_cidr_block = Some(value.into());
        self
    }

    /// Set is_oracle_gua_allocation_enabled (unwraps Option)
    pub fn with_is_oracle_gua_allocation_enabled(mut self, value: bool) -> Self {
        self.is_oracle_gua_allocation_enabled = Some(value);
        self
    }

    /// Set byoipv6_cidr_detail (unwraps Option)
    pub fn with_byoipv6_cidr_detail(mut self, value: Byoipv6CidrDetails) -> Self {
        self.byoipv6_cidr_detail = Some(value);
        self
    }
}

impl Default for AddVcnIpv6CidrDetails {
    fn default() -> Self {
        Self::new()
    }
}
