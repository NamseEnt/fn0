use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Optional mutable instance options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceOptions {
    /// Whether to disable the legacy (/v1) instance metadata service endpoints. Customers who have migrated to /v2 should set this to true for added security. Default is false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub are_legacy_imds_endpoints_disabled: Option<bool>,
}

impl InstanceOptions {
    /// Create a new InstanceOptions
    pub fn new() -> Self {
        Self {
            are_legacy_imds_endpoints_disabled: None,
        }
    }

    /// Set are_legacy_imds_endpoints_disabled
    pub fn set_are_legacy_imds_endpoints_disabled(mut self, value: Option<bool>) -> Self {
        self.are_legacy_imds_endpoints_disabled = value;
        self
    }

    /// Set are_legacy_imds_endpoints_disabled (unwraps Option)
    pub fn with_are_legacy_imds_endpoints_disabled(mut self, value: bool) -> Self {
        self.are_legacy_imds_endpoints_disabled = Some(value);
        self
    }
}

impl Default for InstanceOptions {
    fn default() -> Self {
        Self::new()
    }
}
