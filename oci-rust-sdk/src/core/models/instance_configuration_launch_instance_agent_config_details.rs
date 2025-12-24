use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Configuration options for the Oracle Cloud Agent software running on the instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationLaunchInstanceAgentConfigDetails {
    /// Whether Oracle Cloud Agent can gather performance metrics and monitor the instance using the monitoring plugins. Default value is false (monitoring plugins are enabled). <p> These are the monitoring plugins: Compute Instance Monitoring and Custom Logs Monitoring. <p> The monitoring plugins are controlled by this parameter and by the per-plugin configuration in the {@code pluginsConfig} object. <p> - If {@code isMonitoringDisabled} is true, all of the monitoring plugins are disabled, regardless of the per-plugin configuration. - If {@code isMonitoringDisabled} is false, all of the monitoring plugins are enabled. You can optionally disable individual monitoring plugins by providing a value in the {@code pluginsConfig} object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_monitoring_disabled: Option<bool>,

    /// Whether Oracle Cloud Agent can run all the available management plugins. Default value is false (management plugins are enabled). <p> These are the management plugins: OS Management Service Agent and Compute Instance Run Command. <p> The management plugins are controlled by this parameter and by the per-plugin configuration in the {@code pluginsConfig} object. <p> - If {@code isManagementDisabled} is true, all of the management plugins are disabled, regardless of the per-plugin configuration. - If {@code isManagementDisabled} is false, all of the management plugins are enabled. You can optionally disable individual management plugins by providing a value in the {@code pluginsConfig} object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_management_disabled: Option<bool>,

    /// Whether Oracle Cloud Agent can run all the available plugins. This includes the management and monitoring plugins. <p> To get a list of available plugins, use the {@link #listInstanceagentAvailablePlugins(ListInstanceagentAvailablePluginsRequest) listInstanceagentAvailablePlugins} operation in the Oracle Cloud Agent API. For more information about the available plugins, see [Managing Plugins with Oracle Cloud Agent](https://docs.oracle.com/iaas/Content/Compute/Tasks/manage-plugins.htm).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub are_all_plugins_disabled: Option<bool>,

    /// The configuration of plugins associated with this instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugins_config: Option<Vec<InstanceAgentPluginConfigDetails>>,
}

impl InstanceConfigurationLaunchInstanceAgentConfigDetails {
    /// Create a new InstanceConfigurationLaunchInstanceAgentConfigDetails
    pub fn new() -> Self {
        Self {
            is_monitoring_disabled: None,

            is_management_disabled: None,

            are_all_plugins_disabled: None,

            plugins_config: None,
        }
    }

    /// Set is_monitoring_disabled
    pub fn set_is_monitoring_disabled(mut self, value: Option<bool>) -> Self {
        self.is_monitoring_disabled = value;
        self
    }

    /// Set is_management_disabled
    pub fn set_is_management_disabled(mut self, value: Option<bool>) -> Self {
        self.is_management_disabled = value;
        self
    }

    /// Set are_all_plugins_disabled
    pub fn set_are_all_plugins_disabled(mut self, value: Option<bool>) -> Self {
        self.are_all_plugins_disabled = value;
        self
    }

    /// Set plugins_config
    pub fn set_plugins_config(
        mut self,
        value: Option<Vec<InstanceAgentPluginConfigDetails>>,
    ) -> Self {
        self.plugins_config = value;
        self
    }

    /// Set is_monitoring_disabled (unwraps Option)
    pub fn with_is_monitoring_disabled(mut self, value: bool) -> Self {
        self.is_monitoring_disabled = Some(value);
        self
    }

    /// Set is_management_disabled (unwraps Option)
    pub fn with_is_management_disabled(mut self, value: bool) -> Self {
        self.is_management_disabled = Some(value);
        self
    }

    /// Set are_all_plugins_disabled (unwraps Option)
    pub fn with_are_all_plugins_disabled(mut self, value: bool) -> Self {
        self.are_all_plugins_disabled = Some(value);
        self
    }

    /// Set plugins_config (unwraps Option)
    pub fn with_plugins_config(mut self, value: Vec<InstanceAgentPluginConfigDetails>) -> Self {
        self.plugins_config = Some(value);
        self
    }
}

impl Default for InstanceConfigurationLaunchInstanceAgentConfigDetails {
    fn default() -> Self {
        Self::new()
    }
}
