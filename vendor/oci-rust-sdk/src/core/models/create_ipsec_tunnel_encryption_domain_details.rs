use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Request to enable a multi-encryption domain policy on the IPSec tunnel. There can't be more than 50 security associations in use at one time. See [Encryption domain for policy-based tunnels](https://docs.oracle.com/iaas/Content/Network/Tasks/ipsecencryptiondomains.htm#spi_policy_based_tunnel) for more.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateIPSecTunnelEncryptionDomainDetails {
    /// Lists IPv4 or IPv6-enabled subnets in your Oracle tenancy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oracle_traffic_selector: Option<Vec<String>>,

    /// Lists IPv4 or IPv6-enabled subnets in your on-premises network.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpe_traffic_selector: Option<Vec<String>>,
}

impl CreateIPSecTunnelEncryptionDomainDetails {
    /// Create a new CreateIPSecTunnelEncryptionDomainDetails
    pub fn new() -> Self {
        Self {
            oracle_traffic_selector: None,

            cpe_traffic_selector: None,
        }
    }

    /// Set oracle_traffic_selector
    pub fn set_oracle_traffic_selector(mut self, value: Option<Vec<String>>) -> Self {
        self.oracle_traffic_selector = value;
        self
    }

    /// Set cpe_traffic_selector
    pub fn set_cpe_traffic_selector(mut self, value: Option<Vec<String>>) -> Self {
        self.cpe_traffic_selector = value;
        self
    }

    /// Set oracle_traffic_selector (unwraps Option)
    pub fn with_oracle_traffic_selector(mut self, value: Vec<String>) -> Self {
        self.oracle_traffic_selector = Some(value);
        self
    }

    /// Set cpe_traffic_selector (unwraps Option)
    pub fn with_cpe_traffic_selector(mut self, value: Vec<String>) -> Self {
        self.cpe_traffic_selector = Some(value);
        self
    }
}

impl Default for CreateIPSecTunnelEncryptionDomainDetails {
    fn default() -> Self {
        Self::new()
    }
}
