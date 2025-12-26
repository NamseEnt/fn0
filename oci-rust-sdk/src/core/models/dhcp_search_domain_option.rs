use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// DHCP option for specifying a search domain name for DNS queries. For more information, see [DNS in Your Virtual Cloud Network](https://docs.oracle.com/iaas/Content/Network/Concepts/dns.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DhcpSearchDomainOption {
    /// A single search domain name according to [RFC 952](https://tools.ietf.org/html/rfc952) and [RFC 1123](https://tools.ietf.org/html/rfc1123). During a DNS query, the OS will append this search domain name to the value being queried. <p> If you set {@link DhcpDnsOption} to {@code VcnLocalPlusInternet}, and you assign a DNS label to the VCN during creation, the search domain name in the VCN's default set of DHCP options is automatically set to the VCN domain (for example, {@code vcn1.oraclevcn.com}). <p> If you don't want to use a search domain name, omit this option from the set of DHCP options. Do not include this option with an empty list of search domain names, or with an empty string as the value for any search domain name.
    pub search_domain_names: Vec<String>,

    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for DhcpSearchDomainOption
pub struct DhcpSearchDomainOptionRequired {
    /// A single search domain name according to [RFC 952](https://tools.ietf.org/html/rfc952) and [RFC 1123](https://tools.ietf.org/html/rfc1123). During a DNS query, the OS will append this search domain name to the value being queried. <p> If you set {@link DhcpDnsOption} to {@code VcnLocalPlusInternet}, and you assign a DNS label to the VCN during creation, the search domain name in the VCN's default set of DHCP options is automatically set to the VCN domain (for example, {@code vcn1.oraclevcn.com}). <p> If you don't want to use a search domain name, omit this option from the set of DHCP options. Do not include this option with an empty list of search domain names, or with an empty string as the value for any search domain name.
    pub search_domain_names: Vec<String>,

    pub r#type: String,
}

impl DhcpSearchDomainOption {
    /// Create a new DhcpSearchDomainOption with required fields
    pub fn new(required: DhcpSearchDomainOptionRequired) -> Self {
        Self {
            search_domain_names: required.search_domain_names,

            r#type: required.r#type,
        }
    }

    /// Set search_domain_names
    pub fn set_search_domain_names(mut self, value: Vec<String>) -> Self {
        self.search_domain_names = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
