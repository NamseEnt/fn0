use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The configuration of plugins associated with this instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceAgentPluginConfigDetails {
    /// The plugin name. To get a list of available plugins, use the {@link #listInstanceagentAvailablePlugins(ListInstanceagentAvailablePluginsRequest) listInstanceagentAvailablePlugins} operation in the Oracle Cloud Agent API. For more information about the available plugins, see [Managing Plugins with Oracle Cloud Agent](https://docs.oracle.com/iaas/Content/Compute/Tasks/manage-plugins.htm).
    pub name: String,

    /// Whether the plugin should be enabled or disabled. <p> To enable the monitoring and management plugins, the {@code isMonitoringDisabled} and {@code isManagementDisabled} attributes must also be set to false.
    pub desired_state: InstanceAgentPluginConfigDetailsDesiredState,
}

/// Required fields for InstanceAgentPluginConfigDetails
pub struct InstanceAgentPluginConfigDetailsRequired {
    /// The plugin name. To get a list of available plugins, use the {@link #listInstanceagentAvailablePlugins(ListInstanceagentAvailablePluginsRequest) listInstanceagentAvailablePlugins} operation in the Oracle Cloud Agent API. For more information about the available plugins, see [Managing Plugins with Oracle Cloud Agent](https://docs.oracle.com/iaas/Content/Compute/Tasks/manage-plugins.htm).
    pub name: String,

    /// Whether the plugin should be enabled or disabled. <p> To enable the monitoring and management plugins, the {@code isMonitoringDisabled} and {@code isManagementDisabled} attributes must also be set to false.
    pub desired_state: InstanceAgentPluginConfigDetailsDesiredState,
}

impl InstanceAgentPluginConfigDetails {
    /// Create a new InstanceAgentPluginConfigDetails with required fields
    pub fn new(required: InstanceAgentPluginConfigDetailsRequired) -> Self {
        Self {
            name: required.name,

            desired_state: required.desired_state,
        }
    }

    /// Set name
    pub fn set_name(mut self, value: String) -> Self {
        self.name = value;
        self
    }

    /// Set desired_state
    pub fn set_desired_state(
        mut self,
        value: InstanceAgentPluginConfigDetailsDesiredState,
    ) -> Self {
        self.desired_state = value;
        self
    }
}
