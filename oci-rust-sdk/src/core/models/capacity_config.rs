use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies the capacity configs that the Dedicated Virtual Machine Host (DVMH) Shape could support.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapacityConfig {
    /// The name of each capacity config.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity_config_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_capabilities: Option<SupportedCapabilities>,

    /// Whether this capacity config is the default config.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,

    /// A list of total CPU and memory per capacity bucket.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity_bins: Option<Vec<CapacityBinPreview>>,
}

impl CapacityConfig {
    /// Create a new CapacityConfig
    pub fn new() -> Self {
        Self {
            capacity_config_name: None,

            supported_capabilities: None,

            is_default: None,

            capacity_bins: None,
        }
    }

    /// Set capacity_config_name
    pub fn set_capacity_config_name(mut self, value: Option<String>) -> Self {
        self.capacity_config_name = value;
        self
    }

    /// Set supported_capabilities
    pub fn set_supported_capabilities(mut self, value: Option<SupportedCapabilities>) -> Self {
        self.supported_capabilities = value;
        self
    }

    /// Set is_default
    pub fn set_is_default(mut self, value: Option<bool>) -> Self {
        self.is_default = value;
        self
    }

    /// Set capacity_bins
    pub fn set_capacity_bins(mut self, value: Option<Vec<CapacityBinPreview>>) -> Self {
        self.capacity_bins = value;
        self
    }

    /// Set capacity_config_name (unwraps Option)
    pub fn with_capacity_config_name(mut self, value: impl Into<String>) -> Self {
        self.capacity_config_name = Some(value.into());
        self
    }

    /// Set supported_capabilities (unwraps Option)
    pub fn with_supported_capabilities(mut self, value: SupportedCapabilities) -> Self {
        self.supported_capabilities = Some(value);
        self
    }

    /// Set is_default (unwraps Option)
    pub fn with_is_default(mut self, value: bool) -> Self {
        self.is_default = Some(value);
        self
    }

    /// Set capacity_bins (unwraps Option)
    pub fn with_capacity_bins(mut self, value: Vec<CapacityBinPreview>) -> Self {
        self.capacity_bins = Some(value);
        self
    }
}

impl Default for CapacityConfig {
    fn default() -> Self {
        Self::new()
    }
}
