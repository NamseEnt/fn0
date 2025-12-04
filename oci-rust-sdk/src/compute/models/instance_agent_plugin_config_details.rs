use serde::{Deserialize, Serialize};

/// The configuration of plugins associated with this instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceAgentPluginConfigDetails {
    /// The plugin name. To get a list of available plugins, use the
    /// ListInstanceagentAvailablePlugins operation in the Oracle Cloud Agent API.
    pub name: String,

    /// Whether the plugin should be enabled or disabled.
    /// To enable the monitoring and management plugins, the isMonitoringDisabled and
    /// isManagementDisabled attributes must also be set to false.
    pub desired_state: DesiredState,
}

/// The desired state of the plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DesiredState {
    Enabled,
    Disabled,
}

impl InstanceAgentPluginConfigDetails {
    pub fn new(name: impl Into<String>, desired_state: DesiredState) -> Self {
        Self {
            name: name.into(),
            desired_state,
        }
    }

    pub fn enabled(name: impl Into<String>) -> Self {
        Self::new(name, DesiredState::Enabled)
    }

    pub fn disabled(name: impl Into<String>) -> Self {
        Self::new(name, DesiredState::Disabled)
    }
}
