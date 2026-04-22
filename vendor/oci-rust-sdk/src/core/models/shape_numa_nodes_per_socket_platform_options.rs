use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Configuration options for NUMA nodes per socket.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeNumaNodesPerSocketPlatformOptions {
    /// The supported values for this platform configuration property.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_values: Option<Vec<ShapeNumaNodesPerSocketPlatformOptionsAllowedValues>>,

    /// The default NUMA nodes per socket configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
}

impl ShapeNumaNodesPerSocketPlatformOptions {
    /// Create a new ShapeNumaNodesPerSocketPlatformOptions
    pub fn new() -> Self {
        Self {
            allowed_values: None,

            default_value: None,
        }
    }

    /// Set allowed_values
    pub fn set_allowed_values(
        mut self,
        value: Option<Vec<ShapeNumaNodesPerSocketPlatformOptionsAllowedValues>>,
    ) -> Self {
        self.allowed_values = value;
        self
    }

    /// Set default_value
    pub fn set_default_value(mut self, value: Option<String>) -> Self {
        self.default_value = value;
        self
    }

    /// Set allowed_values (unwraps Option)
    pub fn with_allowed_values(
        mut self,
        value: Vec<ShapeNumaNodesPerSocketPlatformOptionsAllowedValues>,
    ) -> Self {
        self.allowed_values = Some(value);
        self
    }

    /// Set default_value (unwraps Option)
    pub fn with_default_value(mut self, value: impl Into<String>) -> Self {
        self.default_value = Some(value.into());
        self
    }
}

impl Default for ShapeNumaNodesPerSocketPlatformOptions {
    fn default() -> Self {
        Self::new()
    }
}
