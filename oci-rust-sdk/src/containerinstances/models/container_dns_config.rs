use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// DNS settings for containers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerDnsConfig {
    /// IP address of the name server..
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nameservers: Option<Vec<String>>,

    /// Search list for hostname lookup.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub searches: Option<Vec<String>>,

    /// Options allows certain internal resolver variables to be modified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
}

impl ContainerDnsConfig {
    /// Create a new ContainerDnsConfig
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

impl Default for ContainerDnsConfig {
    fn default() -> Self {
        Self::new()
    }
}
