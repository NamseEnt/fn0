use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Configuration information used by the encryption domain policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncryptionDomainConfig {
    /// Lists IPv4 or IPv6-enabled subnets in your Oracle tenancy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oracle_traffic_selector: Option<Vec<String>>,

    /// Lists IPv4 or IPv6-enabled subnets in your on-premises network.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpe_traffic_selector: Option<Vec<String>>,
}

impl EncryptionDomainConfig {
    /// Create a new EncryptionDomainConfig
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

impl Default for EncryptionDomainConfig {
    fn default() -> Self {
        Self::new()
    }
}
