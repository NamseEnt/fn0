use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// DHCP option for specifying how DNS (hostname resolution) is handled in the subnets in the VCN. For more information, see [DNS in Your Virtual Cloud Network](https://docs.oracle.com/iaas/Content/Network/Concepts/dns.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DhcpDnsOption {
    /// * **VcnLocal:** Reserved for future use. <p> **VcnLocalPlusInternet:** Also referred to as \"Internet and VCN Resolver\". Instances can resolve internet hostnames (no internet gateway is required), and can resolve hostnames of instances in the VCN. This is the default value in the default set of DHCP options in the VCN. For the Internet and VCN Resolver to work across the VCN, there must also be a DNS label set for the VCN, a DNS label set for each subnet, and a hostname for each instance. The Internet and VCN Resolver also enables reverse DNS lookup, which lets you determine the hostname corresponding to the private IP address. For more information, see [DNS in Your Virtual Cloud Network](https://docs.oracle.com/iaas/Content/Network/Concepts/dns.htm). <p> **CustomDnsServer:** Instances use a DNS server of your choice (three maximum).
    pub server_type: DhcpDnsOptionServerType,

    #[serde(rename = "type")]
    pub r#type: String,

    /// If you set {@code serverType} to {@code CustomDnsServer}, specify the IP address of at least one DNS server of your choice (three maximum).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_dns_servers: Option<Vec<String>>,
}

/// Required fields for DhcpDnsOption
pub struct DhcpDnsOptionRequired {
    /// * **VcnLocal:** Reserved for future use. <p> **VcnLocalPlusInternet:** Also referred to as \"Internet and VCN Resolver\". Instances can resolve internet hostnames (no internet gateway is required), and can resolve hostnames of instances in the VCN. This is the default value in the default set of DHCP options in the VCN. For the Internet and VCN Resolver to work across the VCN, there must also be a DNS label set for the VCN, a DNS label set for each subnet, and a hostname for each instance. The Internet and VCN Resolver also enables reverse DNS lookup, which lets you determine the hostname corresponding to the private IP address. For more information, see [DNS in Your Virtual Cloud Network](https://docs.oracle.com/iaas/Content/Network/Concepts/dns.htm). <p> **CustomDnsServer:** Instances use a DNS server of your choice (three maximum).
    pub server_type: DhcpDnsOptionServerType,

    pub r#type: String,
}

impl DhcpDnsOption {
    /// Create a new DhcpDnsOption with required fields
    pub fn new(required: DhcpDnsOptionRequired) -> Self {
        Self {
            server_type: required.server_type,

            r#type: required.r#type,

            custom_dns_servers: None,
        }
    }

    /// Set custom_dns_servers
    pub fn set_custom_dns_servers(mut self, value: Option<Vec<String>>) -> Self {
        self.custom_dns_servers = value;
        self
    }

    /// Set server_type
    pub fn set_server_type(mut self, value: DhcpDnsOptionServerType) -> Self {
        self.server_type = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }

    /// Set custom_dns_servers (unwraps Option)
    pub fn with_custom_dns_servers(mut self, value: Vec<String>) -> Self {
        self.custom_dns_servers = Some(value);
        self
    }
}
