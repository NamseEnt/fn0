use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Allow customers to define DNS settings for containers. If this is not provided, the containers use the default DNS settings of the subnet.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateContainerDnsConfigDetails {
    /// IP address of a name server that the resolver should query, either an IPv4 address (in dot notation), or an IPv6 address in colon (and possibly dot) notation. If null, uses nameservers from subnet dhcpDnsOptions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nameservers: Option<Vec<String>>,

    /// Search list for host-name lookup. If null, we will use searches from subnet dhcpDnsOptios.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub searches: Option<Vec<String>>,

    /// Options allows certain internal resolver variables to be modified. Options are a list of objects in https://man7.org/linux/man-pages/man5/resolv.conf.5.html. Examples: [\"ndots:n\", \"edns0\"].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
}

impl CreateContainerDnsConfigDetails {
    /// Create a new CreateContainerDnsConfigDetails
    pub fn new() -> Self {
        Self {
            nameservers: None,

            searches: None,

            options: None,
        }
    }

    /// Set nameservers
    pub fn set_nameservers(mut self, value: Option<Vec<String>>) -> Self {
        self.nameservers = value;
        self
    }

    /// Set searches
    pub fn set_searches(mut self, value: Option<Vec<String>>) -> Self {
        self.searches = value;
        self
    }

    /// Set options
    pub fn set_options(mut self, value: Option<Vec<String>>) -> Self {
        self.options = value;
        self
    }

    /// Set nameservers (unwraps Option)
    pub fn with_nameservers(mut self, value: Vec<String>) -> Self {
        self.nameservers = Some(value);
        self
    }

    /// Set searches (unwraps Option)
    pub fn with_searches(mut self, value: Vec<String>) -> Self {
        self.searches = Some(value);
        self
    }

    /// Set options (unwraps Option)
    pub fn with_options(mut self, value: Vec<String>) -> Self {
        self.options = Some(value);
        self
    }
}

impl Default for CreateContainerDnsConfigDetails {
    fn default() -> Self {
        Self::new()
    }
}
