use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Configuration options for the Access Control Service.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeAccessControlServiceEnabledPlatformOptions {
    /// Whether the Access Control Service can be enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_values: Option<Vec<bool>>,

    /// Whether the Access Control Service is enabled by default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default_enabled: Option<bool>,
}

impl ShapeAccessControlServiceEnabledPlatformOptions {
    /// Create a new ShapeAccessControlServiceEnabledPlatformOptions
    pub fn new() -> Self {
        Self {
            allowed_values: None,

            is_default_enabled: None,
        }
    }

    /// Set allowed_values
    pub fn set_allowed_values(mut self, value: Option<Vec<bool>>) -> Self {
        self.allowed_values = value;
        self
    }

    /// Set is_default_enabled
    pub fn set_is_default_enabled(mut self, value: Option<bool>) -> Self {
        self.is_default_enabled = value;
        self
    }

    /// Set allowed_values (unwraps Option)
    pub fn with_allowed_values(mut self, value: Vec<bool>) -> Self {
        self.allowed_values = Some(value);
        self
    }

    /// Set is_default_enabled (unwraps Option)
    pub fn with_is_default_enabled(mut self, value: bool) -> Self {
        self.is_default_enabled = Some(value);
        self
    }
}

impl Default for ShapeAccessControlServiceEnabledPlatformOptions {
    fn default() -> Self {
        Self::new()
    }
}
