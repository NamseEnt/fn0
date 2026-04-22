use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Information used to create the proxy configuration for a management station.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProxyConfigurationDetails {
    /// Indicates if the proxy should be enabled or disabled. Default is enabled.
    pub is_enabled: bool,

    /// List of hosts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hosts: Option<Vec<String>>,

    /// Listening port used for the proxy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<String>,

    /// The URL the proxy will forward to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forward: Option<String>,
}

/// Required fields for CreateProxyConfigurationDetails
pub struct CreateProxyConfigurationDetailsRequired {
    /// Indicates if the proxy should be enabled or disabled. Default is enabled.
    pub is_enabled: bool,
}

impl CreateProxyConfigurationDetails {
    /// Create a new CreateProxyConfigurationDetails with required fields
    pub fn new(required: CreateProxyConfigurationDetailsRequired) -> Self {
        Self {
            is_enabled: required.is_enabled,

            hosts: None,

            port: None,

            forward: None,
        }
    }

    /// Set is_enabled
    pub fn set_is_enabled(mut self, value: bool) -> Self {
        self.is_enabled = value;
        self
    }

    /// Set hosts
    pub fn set_hosts(mut self, value: Option<Vec<String>>) -> Self {
        self.hosts = value;
        self
    }

    /// Set port
    pub fn set_port(mut self, value: Option<String>) -> Self {
        self.port = value;
        self
    }

    /// Set forward
    pub fn set_forward(mut self, value: Option<String>) -> Self {
        self.forward = value;
        self
    }

    /// Set hosts (unwraps Option)
    pub fn with_hosts(mut self, value: Vec<String>) -> Self {
        self.hosts = Some(value);
        self
    }

    /// Set port (unwraps Option)
    pub fn with_port(mut self, value: impl Into<String>) -> Self {
        self.port = Some(value.into());
        self
    }

    /// Set forward (unwraps Option)
    pub fn with_forward(mut self, value: impl Into<String>) -> Self {
        self.forward = Some(value.into());
        self
    }
}
