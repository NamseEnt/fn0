use serde::{Deserialize, Serialize};

/// Optional mutable instance options.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationInstanceOptions {
    /// Whether to disable the legacy (/v1) instance metadata service endpoints.
    /// Default is false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub are_legacy_imds_endpoints_disabled: Option<bool>,
}

impl InstanceConfigurationInstanceOptions {
    pub fn new() -> Self {
        Self {
            are_legacy_imds_endpoints_disabled: None,
        }
    }

    pub fn with_are_legacy_imds_endpoints_disabled(mut self, disabled: bool) -> Self {
        self.are_legacy_imds_endpoints_disabled = Some(disabled);
        self
    }
}

impl Default for InstanceConfigurationInstanceOptions {
    fn default() -> Self {
        Self::new()
    }
}
