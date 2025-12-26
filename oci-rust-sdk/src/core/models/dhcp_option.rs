use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A single DHCP option according to [RFC 1533](https://tools.ietf.org/html/rfc1533). The two options available to use are {@link DhcpDnsOption} and {@link DhcpSearchDomainOption}. For more information, see [DNS in Your Virtual Cloud Network](https://docs.oracle.com/iaas/Content/Network/Concepts/dns.htm) and [DHCP Options](https://docs.oracle.com/iaas/Content/Network/Tasks/managingDHCP.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DhcpOption {
    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for DhcpOption
pub struct DhcpOptionRequired {
    pub r#type: String,
}

impl DhcpOption {
    /// Create a new DhcpOption with required fields
    pub fn new(required: DhcpOptionRequired) -> Self {
        Self {
            r#type: required.r#type,
        }
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
