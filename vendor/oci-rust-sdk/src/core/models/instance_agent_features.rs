use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Oracle Cloud Agent features supported on the image.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceAgentFeatures {
    /// This attribute is not used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_monitoring_supported: Option<bool>,

    /// This attribute is not used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_management_supported: Option<bool>,
}

impl InstanceAgentFeatures {
    /// Create a new InstanceAgentFeatures
    pub fn new() -> Self {
        Self {
            is_monitoring_supported: None,

            is_management_supported: None,
        }
    }

    /// Set is_monitoring_supported
    pub fn set_is_monitoring_supported(mut self, value: Option<bool>) -> Self {
        self.is_monitoring_supported = value;
        self
    }

    /// Set is_management_supported
    pub fn set_is_management_supported(mut self, value: Option<bool>) -> Self {
        self.is_management_supported = value;
        self
    }

    /// Set is_monitoring_supported (unwraps Option)
    pub fn with_is_monitoring_supported(mut self, value: bool) -> Self {
        self.is_monitoring_supported = Some(value);
        self
    }

    /// Set is_management_supported (unwraps Option)
    pub fn with_is_management_supported(mut self, value: bool) -> Self {
        self.is_management_supported = Some(value);
        self
    }
}

impl Default for InstanceAgentFeatures {
    fn default() -> Self {
        Self::new()
    }
}
