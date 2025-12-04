use serde::{Deserialize, Serialize};

/// Configuration options for the Oracle Cloud Agent software running on the instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationLaunchInstanceAgentConfigDetails {
    /// Whether Oracle Cloud Agent can gather performance metrics and monitor the instance.
    /// Default value is false (monitoring plugins are enabled).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_monitoring_disabled: Option<bool>,

    /// Whether Oracle Cloud Agent can run all the available management plugins.
    /// Default value is false (management plugins are enabled).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_management_disabled: Option<bool>,

    /// Whether Oracle Cloud Agent can run all the available plugins.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub are_all_plugins_disabled: Option<bool>,

    /// The configuration of plugins associated with this instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugins_config: Option<Vec<super::InstanceAgentPluginConfigDetails>>,
}

impl InstanceConfigurationLaunchInstanceAgentConfigDetails {
    pub fn new() -> Self {
        Self {
            is_monitoring_disabled: None,
            is_management_disabled: None,
            are_all_plugins_disabled: None,
            plugins_config: None,
        }
    }

    pub fn with_is_monitoring_disabled(mut self, disabled: bool) -> Self {
        self.is_monitoring_disabled = Some(disabled);
        self
    }

    pub fn with_is_management_disabled(mut self, disabled: bool) -> Self {
        self.is_management_disabled = Some(disabled);
        self
    }

    pub fn with_are_all_plugins_disabled(mut self, disabled: bool) -> Self {
        self.are_all_plugins_disabled = Some(disabled);
        self
    }
}

impl Default for InstanceConfigurationLaunchInstanceAgentConfigDetails {
    fn default() -> Self {
        Self::new()
    }
}
