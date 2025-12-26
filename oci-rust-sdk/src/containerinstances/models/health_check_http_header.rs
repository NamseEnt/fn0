use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Container Http headers for Http health check.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheckHttpHeader {
    /// Container HTTP header Key.
    pub name: String,

    /// Container HTTP header value.
    pub value: String,
}

/// Required fields for HealthCheckHttpHeader
pub struct HealthCheckHttpHeaderRequired {
    /// Container HTTP header Key.
    pub name: String,

    /// Container HTTP header value.
    pub value: String,
}

impl HealthCheckHttpHeader {
    /// Create a new HealthCheckHttpHeader with required fields
    pub fn new(required: HealthCheckHttpHeaderRequired) -> Self {
        Self {
            name: required.name,

            value: required.value,
        }
    }

    /// Set name
    pub fn set_name(mut self, value: String) -> Self {
        self.name = value;
        self
    }

    /// Set value
    pub fn set_value(mut self, value: String) -> Self {
        self.value = value;
        self
    }
}
